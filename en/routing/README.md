<!-- doc-id: CIRNETRULES-ROUTING -->
<!-- revision: 1 -->
<a id="routing"></a>

# Local workspace and routing

`workspace-discover` inspects bounded workspace metadata and proposes candidates.
Use `workspace-bind` with a registered repo ID, a workspace-relative path and an
operator label. It validates the Git root, admin and common directories and
the anchor commit. Replacing a binding requires the previous identity.
`workspace-health` checks one binding. Remote URLs are optional on this path.
Onboard a new repo by reviewing its profile, ownership, classification and
typed relations in the registry, then binding an actual worktree locally.

Use exact files for task scopes. Module cards and direct dependencies explain
possible impact. Explicit read scope limits which sources are opened, and the
write grant is checked separately. The CLI labels product and task material as
data. Authority comes from governance instruction entries. A small UI task
does not resolve unrelated repositories. WORK never calls release validators.

`task-context` emits the English entrypoint, mini-spec, mini-plan, selected
skill and exact product sources. A stable session token requests deltas.
Change the token for a new or compacted session. Dirty, missing and untracked
selected paths are checked by content. The cache stores digests, applies
configured retention and can be pruned with `context-prune`.
Context reports source bytes, files, repos, hits and elapsed time.
