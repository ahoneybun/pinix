use std::time::Duration;

use indicatif::{ProgressBar, ProgressFinish, ProgressStyle};

use crate::action::{Action, BuildStepId, StartFields};
use crate::handlers::logs::LogHandler;
use crate::state::{Handler, HandlerResult, State};
use crate::style::template_style;
use crate::util::indicatif_ext::ProgressBarExt;

fn build_style(size: u16) -> ProgressStyle {
    template_style(size, true, |_| "{msg} {spinner} {wide_bar}", |_| "").tick_chars("…  ")
}

pub fn handle_new_unknown(state: &mut State, action: &Action) -> anyhow::Result<HandlerResult> {
    if let Action::Start {
        start_type: StartFields::Unknown,
        id,
        text,
        ..
    } = action
    {
        let handler = Unknown::new(*id, text, state);
        state.plug(handler);
        state.plug(LogHandler::new(*id));
    };

    Ok(HandlerResult::Continue)
}

struct Unknown {
    id: BuildStepId,
    progress: ProgressBar,
}

impl Unknown {
    fn new(id: BuildStepId, text: &str, state: &mut State) -> Self {
        let first_char = text.chars().next().unwrap_or(' ').to_ascii_uppercase();
        let first_char_len = text.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
        let message = format!("{first_char}{}", &text[first_char_len..]);

        let progress = ProgressBar::new_spinner()
            .with_style(build_style(state.term_size))
            .with_message(message)
            .with_finish(ProgressFinish::AndClear);

        let progress = state.add(progress);
        progress.spawn_steady_tick(Duration::from_secs(1));

        Self { id, progress }
    }
}

impl Handler for Unknown {
    fn on_action(&mut self, _state: &mut State, action: &Action) -> anyhow::Result<HandlerResult> {
        if matches!(action , Action::Stop { id } if *id == self.id) {
            Ok(HandlerResult::Close)
        } else {
            Ok(HandlerResult::Continue)
        }
    }

    fn on_resize(&mut self, state: &mut State) -> anyhow::Result<()> {
        self.progress.set_style(build_style(state.term_size));
        Ok(())
    }
}
