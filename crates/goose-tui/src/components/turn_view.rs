use iocraft::prelude::*;

use crate::colors::*;
use crate::components::tool_call_card::{ToolCallCard, ToolCallCompact};
use crate::types::{ToolStatus, Turn};

#[derive(Default, Props)]
pub struct TurnViewProps {
    pub turn: Option<Turn>,
    pub expanded_tool_call: Option<String>,
    /// Whether the agent is currently streaming into this turn.
    pub active: bool,
    pub status: String,
    pub width: u16,
    /// Terminal height in rows; used to compute the visible line window.
    pub height: u16,
    /// Lines scrolled up from the bottom; 0 = show latest output.
    pub scroll_offset: i32,
}

#[component]
pub fn TurnView(props: &TurnViewProps) -> impl Into<AnyElement<'static>> {
    let Some(turn) = &props.turn else {
        return element!(View);
    };

    // "Featured" tool call: last non-success one, falling back to the last overall.
    let featured_id = turn
        .tool_call_order
        .iter()
        .rev()
        .find(|id| {
            turn.tool_calls
                .get(*id)
                .is_some_and(|tc| tc.status != ToolStatus::Success)
        })
        .or_else(|| turn.tool_call_order.last())
        .cloned();

    // Apply scroll_offset: collect all agent-text lines and select the visible
    // window.  scroll_offset=0 → show the last `height` lines; each +1 shifts
    // the window one line toward the top of the buffer.
    let all_lines: Vec<&str> = turn.agent_text.lines().collect();
    let n = all_lines.len();
    let visible_rows = (props.height as usize).saturating_sub(4); // leave room for tool calls + chrome
    let offset = (props.scroll_offset as usize).min(n.saturating_sub(1));
    let end = n.saturating_sub(offset);
    let start = end.saturating_sub(visible_rows);
    let visible_lines = &all_lines[start..end];

    element! {
        View(flex_direction: FlexDirection::Column, width: props.width) {
            // Tool calls
            #(turn.tool_call_order.iter().map(|id| {
                let tc = turn.tool_calls.get(id).cloned();
                let expanded = props.expanded_tool_call.as_deref() == Some(id.as_str());
                let is_featured = featured_id.as_deref() == Some(id.as_str());

                let el: AnyElement<'static> = if expanded || is_featured {
                    element! {
                        ToolCallCard(key: id.clone(), info: tc, expanded: expanded)
                    }.into()
                } else {
                    element! {
                        ToolCallCompact(key: id.clone(), info: tc)
                    }.into()
                };
                el
            }))

            // Agent text — windowed to the visible rows controlled by scroll_offset.
            #((!turn.agent_text.is_empty()).then(|| element! {
                View(flex_direction: FlexDirection::Column, margin_top: 1, padding_left: 5) {
                    #(visible_lines.iter().map(|line| element! {
                        Text(content: line.to_string(), color: TEXT_PRIMARY)
                    }))
                }
            }))

            // Streaming indicator
            #((props.active && turn.agent_text.is_empty() && turn.tool_call_order.is_empty()).then(|| element! {
                View(padding_left: 5) {
                    Text(content: props.status.clone(), color: TEXT_DIM, italic: true)
                }
            }))
        }
    }
}
