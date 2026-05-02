use serde::{Deserialize, Serialize};

use super::BlockUseAction;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockInteractionDef {
    pub on_use: BlockUseAction,

    #[serde(default)]
    pub sneaking_skips: bool,
}
