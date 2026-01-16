use crate::config::Settings;
use crate::{Ec2CliError, Result};

use super::client::{get_default_vpc, AwsClients, MANAGED_TAG_KEY, MANAGED_TAG_VALUE};

/// Infrastructure resources for ec2-cli
#[derive(Debug, Clone)]
pub struct Infrastructure {
    pub vpc_id: String,
    pub subnet_id: String,
    pub instance_profile_arn: String,
    pub instance_profile_name: String,
}

impl Infrastructure {
    /// Get or create infrastructure for ec2-cli
    /// VPC and subnet come from settings (configured via `config init`)
    /// IAM resources are created if they don't exist
    pub async fn get_or_create(clients: &AwsClients) -> Result<Self> {
        let settings = Settings::load()?;

        // Get VPC ID from settings or use default VPC
        let vpc_id = match settings.vpc_id {
            Some(vpc_id) => vpc_id,
            None => get_default_vpc(clients).await?,
        };

        // Get subnet ID from settings (required)
        let subnet_id = settings
            .subnet_id
            .ok_or(Ec2CliError::SubnetNotConfigured)?;

        // Validate subnet exists and is in the VPC
        validate_subnet(clients, &subnet_id, &vpc_id).await?;

        // Get or create IAM resources
        let (instance_profile_arn, instance_profile_name) =
            get_or_create_iam_resources(clients).await?;

        Ok(Self {
            vpc_id,
            subnet_id,
            instance_profile_arn,
            instance_profile_name,
        })
    }
}

/// Validate that a subnet exists and is in the expected VPC
async fn validate_subnet(clients: &AwsClients, subnet_id: &str, vpc_id: &str) -> Result<()> {
    let subnets = clients
        .ec2
        .describe_subnets()
        .subnet_ids(subnet_id)
        .send()
        .await
        .map_err(Ec2CliError::ec2)?;

    let subnet = subnets
        .subnets()
        .first()
        .ok_or_else(|| Ec2CliError::SubnetNotFound(subnet_id.to_string()))?;

    let actual_vpc = subnet.vpc_id().unwrap_or_default();
    if actual_vpc != vpc_id {
        return Err(Ec2CliError::Config(format!(
            "Subnet {} is in VPC {}, not {}",
            subnet_id, actual_vpc, vpc_id
        )));
    }

    Ok(())
}

/// Get or create IAM role and instance profile for SSM
async fn get_or_create_iam_resources(clients: &AwsClients) -> Result<(String, String)> {
    let role_name = "ec2-cli-instance-role";
    let profile_name = "ec2-cli-instance-profile";

    // Check if role already exists
    let existing_role = clients
        .iam
        .get_role()
        .role_name(role_name)
        .send()
        .await;

    if existing_role.is_err() {
        println!("  Creating IAM role and instance profile...");

        // Create the role
        let assume_role_policy = r#"{
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Principal": {
                        "Service": "ec2.amazonaws.com"
                    },
                    "Action": "sts:AssumeRole"
                }
            ]
        }"#;

        clients
            .iam
            .create_role()
            .role_name(role_name)
            .assume_role_policy_document(assume_role_policy)
            .description("Role for ec2-cli managed instances")
            .tags(
                aws_sdk_iam::types::Tag::builder()
                    .key(MANAGED_TAG_KEY)
                    .value(MANAGED_TAG_VALUE)
                    .build()
                    .map_err(|e| Ec2CliError::Iam(e.to_string()))?,
            )
            .send()
            .await
            .map_err(Ec2CliError::iam)?;

        // Use minimal inline policy for SSM instead of the broader managed policy
        // This restricts permissions to only what's needed for SSM Session Manager
        let ssm_policy = r#"{
            "Version": "2012-10-17",
            "Statement": [
                {
                    "Effect": "Allow",
                    "Action": [
                        "ssm:UpdateInstanceInformation",
                        "ssmmessages:CreateControlChannel",
                        "ssmmessages:CreateDataChannel",
                        "ssmmessages:OpenControlChannel",
                        "ssmmessages:OpenDataChannel",
                        "ec2messages:AcknowledgeMessage",
                        "ec2messages:DeleteMessage",
                        "ec2messages:FailMessage",
                        "ec2messages:GetEndpoint",
                        "ec2messages:GetMessages",
                        "ec2messages:SendReply"
                    ],
                    "Resource": "*"
                }
            ]
        }"#;

        clients
            .iam
            .put_role_policy()
            .role_name(role_name)
            .policy_name("ec2-cli-ssm-policy")
            .policy_document(ssm_policy)
            .send()
            .await
            .map_err(Ec2CliError::iam)?;
    }

    // Check if instance profile exists
    let existing_profile = clients
        .iam
        .get_instance_profile()
        .instance_profile_name(profile_name)
        .send()
        .await;

    let profile_arn = match existing_profile {
        Ok(p) => p
            .instance_profile()
            .ok_or_else(|| Ec2CliError::Iam("No instance profile in response".to_string()))?
            .arn()
            .to_string(),
        Err(_) => {
            // Create instance profile
            let profile = clients
                .iam
                .create_instance_profile()
                .instance_profile_name(profile_name)
                .tags(
                    aws_sdk_iam::types::Tag::builder()
                        .key(MANAGED_TAG_KEY)
                        .value(MANAGED_TAG_VALUE)
                        .build()
                        .map_err(|e| Ec2CliError::Iam(e.to_string()))?,
                )
                .send()
                .await
                .map_err(Ec2CliError::iam)?;

            // Add role to instance profile
            clients
                .iam
                .add_role_to_instance_profile()
                .instance_profile_name(profile_name)
                .role_name(role_name)
                .send()
                .await
                .map_err(Ec2CliError::iam)?;

            // Wait a bit for propagation
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            profile
                .instance_profile()
                .ok_or_else(|| Ec2CliError::Iam("No instance profile in response".to_string()))?
                .arn()
                .to_string()
        }
    };

    Ok((profile_arn, profile_name.to_string()))
}
