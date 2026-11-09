<!-- doc-id: OPENRULES-INTERACTION-STATE -->
<!-- revision: 2 -->
<a id="interaction-state"></a>

# Desktop interaction states

| State | Qt desktop behavior |
| --- | --- |
| Border | Use an outline where it identifies an input or an active keyboard target. Ordinary actions and navigation rows use spacing and surfaces. |
| Background | Preserve the fill of a primary action, input, selected value or content surface. |
| Hover and pressed | Change the action surface briefly and return to the ordinary state after release and pointer exit. |
| Keyboard focus | Display an indicator on the current keyboard target. Mouse focus on an action does not represent a selected route. Text entry keeps its focus affordance. |
| Active route | Derive navigation highlighting from the visible page and the opened conversation ID. A cached selection or pending reply cannot choose the route. |
| Checked value | Preserve visible ticks and radio selections. Theme mode and other preferences remain independent of navigation. |
| Busy | Use the actual request state. Retain the scope and approval semantics of the runtime. |

Use the shared theme and native Qt control geometry for arrows, menu
indicators, and numeric or time stepping. Maintain the accessible name,
keyboard actions and real disabled states when refining a control.
Extend table row backgrounds, hover, and selection through the viewport
width without stretching saved columns or coloring empty space below rows.

The Settings category list routes to application functions and searches local
setting names. It does not query repository content. Reduced motion belongs to
the revisioned presentation preferences. Toasts stack down from the upper
right in Conversation and up from the lower right on other pages. Keep
modal decisions separate from notifications and reserve space for the chat
composer and the Conversation Actions button.

[Operator guide](dashboard.md) · [Runtime architecture](dashboard-architecture.md)
