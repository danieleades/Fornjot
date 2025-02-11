use std::thread;

use crossbeam_channel::{Receiver, SendError, Sender};

use crate::{Error, Evaluation, Model};

/// Evaluates a model in a background thread
pub struct Evaluator {
    trigger_tx: Sender<TriggerEvaluation>,
    event_rx: Receiver<ModelEvent>,
}

impl Evaluator {
    /// Create an `Evaluator` from a model
    pub fn from_model(model: Model) -> Self {
        let (event_tx, event_rx) = crossbeam_channel::bounded(0);
        let (trigger_tx, trigger_rx) = crossbeam_channel::bounded(0);

        thread::spawn(move || {
            while let Ok(TriggerEvaluation) = trigger_rx.recv() {
                if let Err(SendError(_)) =
                    event_tx.send(ModelEvent::ChangeDetected)
                {
                    break;
                }

                let evaluation = match model.evaluate() {
                    Ok(evaluation) => evaluation,
                    Err(err) => {
                        if let Err(SendError(_)) =
                            event_tx.send(ModelEvent::Error(err))
                        {
                            break;
                        }
                        continue;
                    }
                };

                if let Err(SendError(_)) =
                    event_tx.send(ModelEvent::Evaluation(evaluation))
                {
                    break;
                };
            }

            // The channel is disconnected, which means this instance of
            // `Evaluator`, as well as all `Sender`s created from it, have been
            // dropped. We're done.
        });

        Self {
            event_rx,
            trigger_tx,
        }
    }

    /// Access a channel for triggering evaluations
    pub fn trigger(&self) -> Sender<TriggerEvaluation> {
        self.trigger_tx.clone()
    }

    /// Access a channel for receiving status updates
    pub fn events(&self) -> Receiver<ModelEvent> {
        self.event_rx.clone()
    }
}

/// Command received by [`Evaluator`] through its channel
pub struct TriggerEvaluation;

/// An event emitted by [`Evaluator`]
pub enum ModelEvent {
    /// A change in the model has been detected
    ChangeDetected,

    /// The model has been evaluated
    Evaluation(Evaluation),

    /// An error
    Error(Error),
}
