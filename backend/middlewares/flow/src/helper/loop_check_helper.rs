//! Infinite loop check helper
//!
//! 具体方案：检查当前流转请求下所有工作流实例是否出现重复的动作，若某个实例出现重复动作则代表当前流转会话触发了无限循环,则停止某个工作流实例的流转。
//!
//! 例如：
//! 需求1从状态A变更为状态B,记作REQ1(A->B)
//! REQ1(A->B)
//!     -> TASK1(A->B)
//!         -> REQ1(B->C)
//!             -> TASK1(B->C)
//!                 -> REQ1(C->A)
//!                     -> TASK1(C->A)
//!                         -> REQ1(A->B) ERROR---- 此处实例REQ1重复触发了A->B的动作，则判定出现无限循环，并停止实例REQ1的A->B的动作，实例REQ1状态停留在A
//!                         -> REQ2(C->A) SUCCESE---- 仅仅停止无限循环的实例，不会影响当前会话内其余的状态流转流程
//!                         -> ... SUCCESE

use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct InstancesTransition {
    inner: HashMap<String, Vec<String>>,
}

impl InstancesTransition {
    pub fn check(&mut self, inst_id: String, tran_id: String) -> bool {
        let tran_ids = self.inner.entry(inst_id).or_insert_with(Vec::new);

        if tran_ids.contains(&tran_id) {
            return false;
        } else {
            tran_ids.push(tran_id.clone());
        }
        true
    }
}
