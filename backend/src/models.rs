use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Member {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub name: String,
    #[serde(rename = "isCreator")]
    pub is_creator: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub creator: String,
    pub members: Vec<Member>,
    #[serde(rename = "isGenerated")]
    pub is_generated: bool,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserGroupResponse {
    pub id: String,
    pub name: String,
    #[serde(rename = "isGenerated")]
    pub is_generated: bool,
    #[serde(rename = "isCreator")]
    pub is_creator: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateGroupInput {
    pub name: String,
    pub creator: String,
    #[serde(rename = "userId")]
    pub user_id: String,
}

#[derive(Debug, Deserialize)]
pub struct JoinGroupInput {
    #[serde(rename = "groupId")]
    pub group_id: String,
    pub name: String,
    #[serde(rename = "userId")]
    pub user_id: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    #[serde(rename = "group_update")]
    GroupUpdate(Group),
    assignment {
        receiver: String,
    },
    #[serde(rename = "assignments_generated")]
    AssignmentsGenerated {
        #[serde(rename = "byUserId")]
        by_user_id: HashMap<String, String>,
        pairs: HashMap<String, String>,
    },
}

pub mod db_models {
    use sqlx::FromRow;

    #[derive(FromRow)]
    pub struct DbGroup {
        pub id: String,
        pub name: String,
        pub creator: String,
        pub is_generated: bool,
    }

    #[derive(FromRow)]
    pub struct DbMember {
        pub group_id: String,
        pub user_id: String,
        pub name: String,
        pub is_creator: bool,
    }

    #[derive(FromRow)]
    pub struct DbAssignment {
        pub group_id: String,
        pub giver_id: String,
        pub receiver_id: String,
    }
}
