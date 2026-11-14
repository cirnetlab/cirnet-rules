# Repository-local instructions template

Template only: review and adapt these prompts before use. Merge useful entries
into existing instructions; never replace a repository's `AGENTS.md` blindly.
Remove prompts that do not apply. Keep common behavior in the shared baseline.

## Identity and ownership

State this repository's purpose, owned interfaces and canonical upstream sources.
Name protected or generated paths and their owning generation commands.

## Architecture and commands

Link the current architecture and relevant nested instructions. Supply actual
setup/build/test commands, working directories and prerequisites for each lane.
State where tests may write and how to allocate output without overwriting evidence.

## Local workflow

State when plans/specs are required and their exact repository-local locations.
Describe documentation/translation conventions and required verification checks.
For bilingual documentation, name the English source and matching Vietnamese paths,
the synchronization check, and required updates before reporting completion.
Keep release, deployment and physical operation authorization separate from coding.

## Evidence limits

Identify candidate versus selected release inputs and unavailable capabilities.
Point to owner-maintained facts; avoid copying facts from consumers or history.
