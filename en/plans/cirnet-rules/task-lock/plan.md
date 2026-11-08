<!-- doc-id: TASK-LOCK-PLAN -->
<!-- revision: 1 -->
<a id="work"></a>

# Task lock regression plan

1. Reproduce lock retention with a duplicated handle and confirm contention is denied.
2. Release the task lock explicitly when its guard leaves scope.
3. Verify concurrent writes, the workflow checks and bilingual guidance.
