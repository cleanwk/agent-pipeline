---
status: accepted
---

# Publish immutable Artifacts with lineage

日志和临时文件不是正式交付；Attempt 只有显式发布满足输出契约的 Artifact 后才能完成。发布时保存不可变内容快照并保留源位置，Agent 或人的后续修改形成新的 Artifact Revision，以 `supersedes` 谱系关联旧版本。每次 Run 通过 Package 声明的 Delivery Slot 汇总重要产出，用户可将选定 Revision 提升为 Workspace Reference，但旧任务内容不会自动污染新任务上下文。
