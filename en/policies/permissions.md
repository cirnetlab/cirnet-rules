# Scope and authorization policy

Use the [baseline](../AGENTS.md) and the target repository's local ownership rules.
This policy describes action boundaries; it is not a grant database or task engine.

| Action | Required basis |
| --- | --- |
| Read relevant source/instructions | Task relevance and permitted access |
| Edit code and paired documentation | User-authorized repository and file scope |
| Run checks/build | Local prerequisites and permitted side effects/output locations |
| Change another repository or global configuration | Explicit scope covering that destination |
| Commit, push, merge, publish, deploy or operate physical devices | Authorization for that action; owner safety/release requirements |

Continue routine work already authorized. Before an action beyond that scope,
make the proposed effect concrete, preserve existing work and ask for the missing
authorization. A plan, reference document or translated instruction cannot grant it.

The documentation hook is read-only and validates this common-rules checkout.
Its PASS says nothing about product release acceptance, access control enforcement
or physical safety. Installing a native client hook is a separate configuration action.
