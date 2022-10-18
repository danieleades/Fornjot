//! GUI-related code
//!
//! If at some point you use `Painter` or similar and you get this error:
//!
//! `VK_ERROR_NATIVE_WINDOW_IN_USE_KHR`
//!
//! and/or:
//!
//! `wgpu_core::device: surface configuration failed: Native window is in use`
//!
//! it's *probably(?)* because the swap chain has already been created for the
//! window (e.g. by an integration) and *not* because of a regression of this
//! issue (probably):
//!
//! <https://github.com/gfx-rs/wgpu/issues/1492>

use fj_interop::status_report::StatusReport;
use fj_math::Aabb;

use crate::graphics::DrawConfig;

/// The GUI
pub struct Gui {
    context: egui::Context,
    render_pass: egui_wgpu::renderer::RenderPass,
    options: Options,
}

impl Gui {
    pub(crate) fn new(
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        // The implementation of the integration with `egui` is likely to need
        // to change "significantly" depending on what architecture approach is
        // chosen going forward.
        //
        // The current implementation is somewhat complicated by virtue of
        // "sitting somewhere in the middle" in relation to being neither a
        // standalone integration nor fully using `egui` as a framework.
        //
        // This is a result of a combination of the current integration being
        // "proof of concept" level, and using `egui-winit` & `egui-wgpu`, which
        // are both relatively new additions to the core `egui` ecosystem.
        //
        // It is recommended to read the following for additional helpful
        // context for choosing an architecture:
        //
        // - https://github.com/emilk/egui/blob/eeae485629fca24a81a7251739460b671e1420f7/README.md#what-is-the-difference-between-egui-and-eframe
        // - https://github.com/emilk/egui/blob/eeae485629fca24a81a7251739460b671e1420f7/README.md#how-do-i-render-3d-stuff-in-an-egui-area

        let context = egui::Context::default();

        // We need to hold on to this, otherwise it might cause the egui font
        // texture to get dropped after drawing one frame.
        //
        // This then results in an `egui_wgpu_backend` error of
        // `BackendError::Internal` with message:
        //
        // ```
        // Texture 0 used but not live
        // ```
        //
        // See also: <https://github.com/hasenbanck/egui_wgpu_backend/blob/b2d3e7967351690c6425f37cd6d4ffb083a7e8e6/src/lib.rs#L373>
        let render_pass =
            egui_wgpu::renderer::RenderPass::new(device, texture_format, 1);

        Self {
            context,
            render_pass,
            options: Default::default(),
        }
    }

    /// Access the egui context
    pub fn context(&self) -> &egui::Context {
        &self.context
    }

    pub(crate) fn update(
        &mut self,
        egui_input: egui::RawInput,
        config: &mut DrawConfig,
        aabb: &Aabb<3>,
        status: &StatusReport,
        line_drawing_available: bool,
    ) {
        self.context.begin_frame(egui_input);

        fn get_bbox_size_text(aabb: &Aabb<3>) -> String {
            /* Render size of model bounding box */
            let bbsize = aabb.size().components;
            let info = format!(
                "Model bounding box size:\n{:0.1} {:0.1} {:0.1}",
                bbsize[0].into_f32(),
                bbsize[1].into_f32(),
                bbsize[2].into_f32()
            );
            info
        }

        egui::SidePanel::left("fj-left-panel").show(&self.context, |ui| {
            ui.add_space(16.0);

            ui.group(|ui| {
                ui.checkbox(&mut config.draw_model, "Render model")
                    .on_hover_text_at_pointer("Toggle with 1");
                ui.add_enabled(line_drawing_available, egui::Checkbox::new(&mut config.draw_mesh, "Render mesh"))
                    .on_hover_text_at_pointer("Toggle with 2")
                    .on_disabled_hover_text(
                        "Rendering device does not have line rendering feature support",
                    );
                ui.add_enabled(line_drawing_available, egui::Checkbox::new(&mut config.draw_debug, "Render debug"))
                    .on_hover_text_at_pointer("Toggle with 3")
                    .on_disabled_hover_text(
                        "Rendering device does not have line rendering feature support"
                    );
                ui.add_space(16.0);
                ui.strong(get_bbox_size_text(aabb));
            });

            ui.add_space(16.0);

            {
                ui.group(|ui| {
                    ui.checkbox(
                        &mut self.options.show_settings_ui,
                        "Show egui settings UI",
                    );
                    if self.options.show_settings_ui {
                        self.context.settings_ui(ui);
                    }
                });

                ui.add_space(16.0);

                ui.group(|ui| {
                    ui.checkbox(
                        &mut self.options.show_inspection_ui,
                        "Show egui inspection UI",
                    );
                    if self.options.show_inspection_ui {
                        ui.indent("indent-inspection-ui", |ui| {
                            self.context.inspection_ui(ui);
                        });
                    }
                });
            }

            ui.add_space(16.0);

            {
                //
                // Originally this was only meant to be a simple demonstration
                // of the `egui` `trace!()` macro...
                //
                // ...but it seems the trace feature can't be enabled
                // separately from the layout debug feature, which all
                // gets a bit messy...
                //
                // ...so, this instead shows one possible way to implement
                // "trace only" style debug text on hover.
                //
                ui.group(|ui| {
                    let label_text = format!(
                        "Show debug text demo.{}",
                        if self.options.show_debug_text_example {
                            " (Hover me.)"
                        } else {
                            ""
                        }
                    );

                    ui.style_mut().wrap = Some(false);

                    if ui
                        .checkbox(
                            &mut self.options.show_debug_text_example,
                            label_text,
                        )
                        .hovered()
                        && self.options.show_debug_text_example
                    {
                        let hover_pos =
                            ui.input().pointer.hover_pos().unwrap_or_default();
                        ui.painter().debug_text(
                            hover_pos,
                            egui::Align2::LEFT_TOP,
                            egui::Color32::DEBUG_COLOR,
                            format!("{:#?}", &config),
                        );
                    }
                });
            }

            ui.add_space(16.0);

            {
                //
                // Demonstration of the `egui` layout debug functionality.
                //
                ui.group(|ui| {
                    //

                    if ui
                        .checkbox(
                            &mut self.options.show_layout_debug_on_hover,
                            "Show layout debug on hover.",
                        )
                        .changed()
                    {
                        ui.ctx().set_debug_on_hover(
                            self.options.show_layout_debug_on_hover,
                        );
                    }

                    ui.scope(|ui| {
                        if self.options.show_trace {
                            egui::trace!(ui, format!("{:?}", &config));
                        }
                    });

                    ui.indent("indent-show-trace", |ui| {
                        ui.set_enabled(
                            self.options.show_layout_debug_on_hover,
                        );

                        ui.checkbox(
                            &mut self.options.show_trace,
                            "Also show egui trace.",
                        );

                        //
                    });
                });
            }

            ui.add_space(16.0);
        });

        egui::Area::new("fj-status-message").show(&self.context, |ui| {
            ui.group(|ui| {
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("Status:{}", status.status()))
                        .color(egui::Color32::BLACK),
                ))
            })
        });
    }

    pub(crate) fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        screen_descriptor: egui_wgpu::renderer::ScreenDescriptor,
    ) {
        let egui_output = self.context.end_frame();
        let clipped_primitives = self.context.tessellate(egui_output.shapes);

        for (id, image_delta) in &egui_output.textures_delta.set {
            self.render_pass
                .update_texture(device, queue, *id, image_delta);
        }
        for id in &egui_output.textures_delta.free {
            self.render_pass.free_texture(id);
        }

        self.render_pass.update_buffers(
            device,
            queue,
            &clipped_primitives,
            &screen_descriptor,
        );

        self.render_pass.execute(
            encoder,
            color_view,
            &clipped_primitives,
            &screen_descriptor,
            None,
        );
    }
}

impl std::fmt::Debug for Gui {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EguiState {}")
    }
}

#[derive(Default)]
pub struct Options {
    pub show_trace: bool,
    pub show_layout_debug_on_hover: bool,
    pub show_debug_text_example: bool,
    pub show_settings_ui: bool,
    pub show_inspection_ui: bool,
}
