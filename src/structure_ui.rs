//! Native D2/D3 commands and transactional dialogs. All mutations use Editor.
use super::*;
use crate::{
    model::Document,
    structure::{Block, ListKind, Picture, Section, Table},
};
#[derive(Clone)]
pub(super) enum StructureDraft {
    Link {
        target: String,
        remove: bool,
    },
    Table {
        value: Table,
        paragraph: Option<usize>,
        row: usize,
        column: usize,
    },
    Picture {
        value: Picture,
        paragraph: Option<usize>,
    },
    Page {
        value: Section,
        new_section: bool,
        remove: bool,
    },
    Flow {
        page_break: bool,
        keep_next: bool,
        keep_lines: bool,
    },
    List {
        level: u8,
        start: u16,
    },
}
pub(super) struct StructureDialog {
    source: Document,
    draft: StructureDraft,
    error: Option<String>,
}
impl StructureDialog {
    fn title(&self) -> &str {
        match self.draft {
            StructureDraft::Link { .. } => "Hyperlink",
            StructureDraft::Table { .. } => "Table",
            StructureDraft::Picture { .. } => "Picture",
            StructureDraft::Page { .. } => "Page setup and sections",
            StructureDraft::Flow { .. } => "Paragraph pagination",
            StructureDraft::List { .. } => "List settings",
        }
    }
    fn apply(&self, editor: &mut crate::commands::Editor) -> Result<()> {
        if editor.document != self.source {
            return Err("Document changed. Cancel and reopen this dialog.".into());
        }
        match &self.draft {
            StructureDraft::Link { target, remove } => {
                editor.set_link(if *remove { None } else { Some(target) })
            }
            StructureDraft::Table {
                value, paragraph, ..
            } => {
                if let Some(p) = paragraph {
                    editor.update_block(*p, Block::Table(value.clone()))
                } else {
                    editor.insert_block(Block::Table(value.clone()))
                }
            }
            StructureDraft::Picture { value, paragraph } => {
                if let Some(p) = paragraph {
                    editor.update_block(*p, Block::Picture(value.clone()))
                } else {
                    editor.insert_block(Block::Picture(value.clone()))
                }
            }
            StructureDraft::Page {
                value,
                new_section,
                remove,
            } => {
                if *remove {
                    editor.remove_section()
                } else {
                    editor.set_page(value.clone(), *new_section)
                }
            }
            StructureDraft::Flow {
                page_break,
                keep_next,
                keep_lines,
            } => editor.set_flow(*page_break, *keep_next, *keep_lines),
            StructureDraft::List { level, start } => {
                let mut doc = editor.document.clone();
                let first = doc.locate(editor.selection.0)?.0;
                for i in crate::structure::paragraph_indices(&doc, editor.selection)? {
                    let list = doc.paragraphs[i].structure.list.get_or_insert(
                        crate::structure::ListItem {
                            kind: ListKind::Decimal,
                            level: 0,
                            restart: None,
                        },
                    );
                    list.level = *level;
                    if i == first {
                        list.restart = Some(*start);
                    }
                }
                editor.commit_structure(doc, "List settings")
            }
        }
    }
    fn show(
        &mut self,
        ctx: &egui::Context,
        editor: &mut crate::commands::Editor,
        regions: &mut Vec<Region>,
    ) -> bool {
        let mut cancel = false;
        let mut apply = false;
        let title = self.title().to_owned();
        egui::Modal::new(Id::new("structure-dialog")).area(egui::Modal::default_area(Id::new("structure-dialog")).fade_in(false)).show(ctx,|ui|{
            ui.set_width((ctx.content_rect().width()-70.).clamp(260.,650.));ui.heading(title);
            egui::ScrollArea::both().max_height((ctx.content_rect().height()-150.).max(100.)).show(ui,|ui|{
                match &mut self.draft {
                    StructureDraft::Link{target,remove}=>{ui.label("Address (http, https or mailto)");let r=ui.text_edit_singleline(target);regions.push(Region{name:"link-target".into(),rect:r.rect});ui.checkbox(remove,"Remove link from selection"); if ui.add_enabled(crate::structure::validate_target(target).is_ok(), Button::new("Open address in browser")).clicked(){ ctx.open_url(egui::OpenUrl::new_tab(target.clone())); }},
                    StructureDraft::Table{value,row,column,..}=>{
                        ui.horizontal_wrapped(|ui|{
                            if ui.add_enabled(value.rows.len()<100,Button::new("Add row")).clicked(){value.rows.insert((*row+1).min(value.rows.len()),vec![String::new();value.widths_points.len()]);}
                            if ui.add_enabled(value.rows.len()>1,Button::new("Remove row")).clicked(){value.rows.remove((*row).min(value.rows.len()-1));*row=(*row).min(value.rows.len()-1);}
                            if ui.add_enabled(value.widths_points.len()<12,Button::new("Add column")).clicked(){let col=(*column+1).min(value.widths_points.len());value.widths_points.insert(col,72);for r in &mut value.rows {r.insert(col,String::new());}}
                            if ui.add_enabled(value.widths_points.len()>1,Button::new("Remove column")).clicked(){let col=(*column).min(value.widths_points.len()-1);value.widths_points.remove(col);for r in &mut value.rows{r.remove(col);}*column=col.min(value.widths_points.len()-1);}
                        });
                        ui.checkbox(&mut value.repeat_header,"Repeat first row on subsequent pages");
                        ui.horizontal(|ui|{ui.label("Selected row / column:");let mut shown_row=*row+1;let mut shown_column=*column+1;ui.add(egui::DragValue::new(&mut shown_row).range(1..=value.rows.len()));ui.add(egui::DragValue::new(&mut shown_column).range(1..=value.widths_points.len()));*row=shown_row-1;*column=shown_column-1;});
                        ui.small("Column widths in points; scaled down together when wider than the page.");
                        egui::Grid::new("table-cells").striped(true).show(ui,|ui|{
                            for width in &mut value.widths_points{ui.add(egui::DragValue::new(width).range(18..=720).suffix(" pt"));}ui.end_row();
                            for (r,cells) in value.rows.iter_mut().enumerate(){for(c,text)in cells.iter_mut().enumerate(){let response=ui.add_sized([130., 48.],egui::TextEdit::multiline(text).id_salt(("cell",r,c)).desired_width(130.).desired_rows(2));regions.push(Region{name:format!("table-cell-{r}-{c}"),rect:response.rect});if response.has_focus(){*row=r;*column=c;}}ui.end_row();}
                        });
                    },
                    StructureDraft::Picture{value,..}=>{
                        ui.label("Anchoring: separate paragraph, in document order.");
                        ui.horizontal(|ui|{ui.label("Width / height:");ui.add(egui::DragValue::new(&mut value.width_points).range(12..=1440).suffix(" pt"));ui.add(egui::DragValue::new(&mut value.height_points).range(12..=1440).suffix(" pt"));});
                        ui.label("Alternative text");ui.text_edit_singleline(&mut value.alt);
                        if let Ok(texture)=crate::object_layout::picture_texture(ctx,value){ui.image((texture.id(),texture.size_vec2().normalized()*180.));}
                    },
                    StructureDraft::Page{value,new_section,remove}=>{
                        ui.horizontal(|ui|{for(label,w,h)in[("Letter",612,792),("A4",595,842),("A5",420,595)]{if ui.button(label).clicked(){value.width_points=w;value.height_points=h;}}if ui.button("Rotate").clicked(){std::mem::swap(&mut value.width_points,&mut value.height_points);}});
                        egui::Grid::new("page-values").num_columns(2).show(ui,|ui|{for(label,v)in[("Width",&mut value.width_points),("Height",&mut value.height_points),("Left margin",&mut value.left_points),("Right margin",&mut value.right_points),("Top margin",&mut value.top_points),("Bottom margin",&mut value.bottom_points)]{ui.label(label);ui.add(egui::DragValue::new(v).range(18..=1440).suffix(" pt"));ui.end_row();}});
                        ui.label("Header");ui.text_edit_singleline(&mut value.header);ui.label("Footer");ui.text_edit_singleline(&mut value.footer);
                        ui.checkbox(&mut value.page_numbers,"Show page number in footer");
                        let mut restart=value.number_start.is_some();ui.horizontal(|ui|{if ui.checkbox(&mut restart,"Restart numbering").changed(){value.number_start=restart.then_some(1);}
if let Some(n)=&mut value.number_start{ui.add(egui::DragValue::new(n).range(1..=u16::MAX));}});
                        ui.separator();ui.checkbox(new_section,"Apply as a new-page section at the current paragraph");ui.checkbox(remove,"Remove section break from current paragraph");
                    },
                    StructureDraft::Flow{page_break,keep_next,keep_lines}=>{ui.checkbox(page_break,"Page break before");ui.checkbox(keep_next,"Keep with next paragraph");ui.checkbox(keep_lines,"Keep lines together");},
                    StructureDraft::List{level,start}=>{ui.horizontal(|ui|{ui.label("Level (0–8)");ui.add(egui::DragValue::new(level).range(0..=8));});ui.horizontal(|ui|{ui.label("Start numbering at");ui.add(egui::DragValue::new(start).range(1..=u16::MAX));});}
                }
            });
            if let Some(error)=&self.error{ui.colored_label(Color32::from_rgb(165,40,40),error);}
            ui.separator();ui.horizontal(|ui|{let r=ui.button("OK");regions.push(Region{name:"structure-ok".into(),rect:r.rect});apply=r.clicked();let r=ui.button("Cancel");regions.push(Region{name:"structure-cancel".into(),rect:r.rect});cancel=r.clicked();});
        });
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            cancel = true;
        }
        if apply {
            match self.apply(editor) {
                Ok(()) => return true,
                Err(e) => self.error = Some(e),
            }
        }
        cancel
    }
}
impl DocsApp {
    pub(super) fn structure_action(&mut self, action: Action, ctx: &egui::Context) {
        let p = self
            .session
            .editor
            .document
            .locate(self.session.editor.selection.0)
            .map(|v| v.0)
            .unwrap_or(0);
        let para = &self.session.editor.document.paragraphs[p];
        let draft = match action {
            Action::Bullets | Action::Numbering => {
                let kind = if action == Action::Bullets {
                    ListKind::Bullet
                } else {
                    ListKind::Decimal
                };
                let toggle = para.structure.list.as_ref().is_some_and(|l| l.kind == kind);
                let r = self
                    .session
                    .editor
                    .set_list(if toggle { None } else { Some(kind) });
                self.fail(r);
                self.sync_view();
                return;
            }
            Action::ListSettings => StructureDraft::List {
                level: para.structure.list.as_ref().map_or(0, |l| l.level),
                start: para
                    .structure
                    .list
                    .as_ref()
                    .and_then(|l| l.restart)
                    .unwrap_or(1),
            },
            Action::Link => StructureDraft::Link {
                target: para
                    .structure
                    .links
                    .iter()
                    .find(|l| {
                        let at = self
                            .session
                            .editor
                            .document
                            .locate(self.session.editor.selection.0)
                            .unwrap()
                            .1;
                        l.start <= at && at < l.end
                    })
                    .map_or("https://".into(), |l| l.target.clone()),
                remove: false,
            },
            Action::Table => StructureDraft::Table {
                value: if let Some(Block::Table(t)) = &para.structure.block {
                    t.clone()
                } else {
                    Table::new(2, 2).unwrap()
                },
                paragraph: matches!(para.structure.block, Some(Block::Table(_))).then_some(p),
                row: 0,
                column: 0,
            },
            Action::Picture => {
                if let Some(Block::Picture(value)) = &para.structure.block {
                    StructureDraft::Picture {
                        value: value.clone(),
                        paragraph: Some(p),
                    }
                } else {
                    let Some(path) = rfd::FileDialog::new()
                        .add_filter("PNG picture", &["png"])
                        .pick_file()
                    else {
                        return;
                    };
                    let result = (|| -> Result<Picture> {
                        let metadata = std::fs::metadata(&path).map_err(|e| e.to_string())?;
                        if metadata.len() > crate::structure::MAX_IMAGE_BYTES as u64 {
                            return Err("Picture exceeds 4 MiB.".into());
                        }
                        let png = std::fs::read(&path).map_err(|e| e.to_string())?;
                        let mut value = Picture {
                            png,
                            width_points: 216,
                            height_points: 144,
                            alt: path
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into(),
                        };
                        value.validate()?;
                        let texture = crate::object_layout::picture_texture(ctx, &value)?;
                        value.height_points = (216. * texture.size_vec2().y / texture.size_vec2().x)
                            .clamp(12., 1440.) as u16;
                        Ok(value)
                    })();
                    match result {
                        Ok(value) => StructureDraft::Picture {
                            value,
                            paragraph: None,
                        },
                        Err(e) => {
                            self.error = Some(e);
                            return;
                        }
                    }
                }
            }
            Action::PageSetup => StructureDraft::Page {
                value: crate::pagination::sections(&self.session.editor.document)[p].clone(),
                new_section: para.structure.section.is_some(),
                remove: false,
            },
            Action::PageFlow => StructureDraft::Flow {
                page_break: para.structure.page_break_before,
                keep_next: para.structure.keep_with_next,
                keep_lines: para.structure.keep_lines,
            },
            Action::PageBreak => {
                let r = self.session.editor.insert_page_break();
                self.fail(r);
                self.sync_view();
                return;
            }
            Action::PrintLayout => {
                self.paged_editing = !self.paged_editing;
                self.page_preview = false;
                self.backstage = false;
                return;
            }
            Action::PrintPreview => {
                self.page_preview = !self.page_preview;
                self.backstage = false;
                return;
            }
            Action::ExportDocx => {
                if let Err(e) = crate::docx::check_export(&self.session.editor.document) {
                    self.error = Some(e);
                    return;
                }
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Export supported DOCX subset")
                    .set_file_name("Document.docx")
                    .add_filter("DOCX", &["docx"])
                    .save_file()
                {
                    match crate::docx::save(&self.session.editor.document, &path) {
                        Ok(()) => {
                            self.output_notice = Some(format!(
                                "DOCX exported: {}. Current save destination unchanged.",
                                path.display()
                            ))
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                return;
            }
            Action::ExportPdf => {
                self.export_pdf(ctx);
                return;
            }
            Action::Print => {
                self.page_preview = true;
                self.backstage = false;
                self.print_requested = true;
                return;
            }
            _ => return,
        };
        self.structure_dialog = Some(StructureDialog {
            source: self.session.editor.document.clone(),
            draft,
            error: None,
        });
        self.focus_document = false;
    }
    pub(super) fn structure_dialogs(&mut self, ctx: &egui::Context) -> bool {
        let Some(mut dialog) = self.structure_dialog.take() else {
            return false;
        };
        if dialog.show(ctx, &mut self.session.editor, &mut self.regions) {
            self.sync_view();
            self.focus_document = true;
        } else {
            self.structure_dialog = Some(dialog);
        }
        true
    }
    pub(super) fn structure_ribbon(&mut self, ui: &mut Ui) {
        let groups: Vec<(&str, Vec<Action>)> = if self.tab == 1 {
            vec![
                ("Pages", vec![Action::PageBreak]),
                ("Tables", vec![Action::Table]),
                ("Illustrations", vec![Action::Picture]),
                ("Links", vec![Action::Link]),
            ]
        } else {
            vec![
                ("Page Setup", vec![Action::PageSetup, Action::PageFlow]),
                (
                    "Paragraph",
                    vec![Action::ParagraphSpacing, Action::TabPositions],
                ),
                ("Output", vec![Action::PrintPreview, Action::ExportPdf]),
            ]
        };
        ui.horizontal_wrapped(|ui| {
            for (label, actions) in groups {
                ui.vertical(|ui| {
                    for action in actions {
                        self.command(ui, action.label(), action, vec2(150., 25.));
                    }
                    ui.small(label);
                });
                ui.separator();
            }
        });
    }
    pub(super) fn object_overlays(
        &mut self,
        ui: &mut Ui,
        galley: &egui::Galley,
        origin: egui::Pos2,
        zoom: f32,
    ) {
        let mut p = 0;
        let mut edit = None;
        for row in &galley.rows {
            if let Some(block) = self
                .session
                .editor
                .document
                .paragraphs
                .get(p)
                .and_then(|p| p.structure.block.as_ref())
            {
                let rect = Rect::from_min_size(origin + row.pos.to_vec2(), row.size);
                if let Block::Picture(picture) = block {
                    if let Ok(texture) = crate::object_layout::picture_texture(ui.ctx(), picture) {
                        ui.painter().image(
                            texture.id(),
                            rect,
                            Rect::from_min_max(egui::Pos2::ZERO, pos2(1., 1.)),
                            Color32::WHITE,
                        );
                    } else {
                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            "Cannot decode picture",
                            FontId::proportional(12. * zoom),
                            Color32::RED,
                        );
                    }
                }
                if ui
                    .interact(rect, Id::new(("object", p)), Sense::click())
                    .double_clicked()
                {
                    edit = Some((p, matches!(block, Block::Table(_))));
                }
            }
            if row.ends_with_newline {
                p += 1;
            }
        }
        if let Some((p, table)) = edit {
            let offset = self.session.editor.document.paragraphs[..p]
                .iter()
                .map(|p| p.styles.len() + 1)
                .sum();
            self.session.editor.selection = (offset, offset + 1);
            self.execute(
                if table {
                    Action::Table
                } else {
                    Action::Picture
                },
                ui.ctx(),
            );
        }
    }
    fn export_pdf(&mut self, _ctx: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .set_file_name("Document.pdf")
            .add_filter("PDF", &["pdf"])
            .save_file()
        else {
            return;
        };
        match crate::pdf_output::export(&self.session.editor.document, &path) {
            Ok(()) => self.output_notice = Some(format!("PDF saved: {}", path.display())),
            Err(e) => self.error = Some(e),
        }
    }
    pub(super) fn page_view(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Back to editing").clicked() {
                self.page_preview = false;
                self.focus_document = true;
            }
            self.command(ui, "Export PDF", Action::ExportPdf, vec2(100., 26.));
        });
        match crate::page_layout::Layout::build(ui.ctx(), &self.session.editor.document) {
            Err(error) => {
                ui.colored_label(Color32::RED, error);
            }
            Ok(layout) => {
                ui.label(format!("{} pages", layout.plan.pages.len()));
                if self.print_requested {
                    self.print_controls(ui);
                }
                egui::ScrollArea::both().show(ui, |ui| {
                    for (index, page) in layout.plan.pages.iter().enumerate() {
                        let scale = (self.zoom / 100.) * 4. / 3.;
                        let size = vec2(
                            f32::from(page.settings.width_points),
                            f32::from(page.settings.height_points),
                        ) * scale;
                        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                        layout.paint(ui.ctx(), ui.painter(), index, rect.min, scale);
                        ui.add_space(16.);
                    }
                });
            }
        }
        if let Some(notice) = &self.output_notice {
            ui.label(notice);
        }
    }
    fn print_controls(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Printer queue:");
            ui.text_edit_singleline(&mut self.printer_queue);
            if ui.button("Refresh printers").clicked() {
                self.output_notice = Some(crate::printing::printers().unwrap_or_else(|e| e));
            }
        });
        ui.label("Print submits this document to the selected system queue. No job is sent by previewing.");
        if ui
            .add_enabled(
                !self.printer_queue.trim().is_empty() && crate::printing::supported(),
                Button::new("Print document"),
            )
            .clicked()
        {
            let result = (|| -> Result<String> {
                let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
                let path = dir.path().join("Document.pdf");
                crate::pdf_output::export(&self.session.editor.document, &path)?;
                crate::printing::submit(&path, &self.printer_queue)
            })();
            match result {
                Ok(message) => self.output_notice = Some(message),
                Err(e) => self.error = Some(e),
            }
        }
        if !crate::printing::supported() {
            ui.label(
                "This platform's print adapter is not available. Export PDF for system printing.",
            );
        }
    }
}

impl DocsApp {
    pub(super) fn paged_document(&mut self, ui: &mut Ui) {
        let model = self
            .tab_ruler_drag
            .as_ref()
            .map(|d| &d.draft.preview)
            .or_else(|| self.ruler_drag.as_ref().map(|d| &d.draft.preview))
            .unwrap_or(&self.session.editor.document)
            .clone();
        let page_layout = match crate::page_layout::Layout::build(ui.ctx(), &model) {
            Ok(l) => l,
            Err(error) => {
                self.page_count = None;
                ui.colored_label(Color32::RED, error);
                if ui.button("Return to continuous editing").clicked() {
                    self.paged_editing = false;
                }
                return;
            }
        };
        self.page_count = Some(page_layout.plan.pages.len());
        let scale = self.zoom / 100. * 4. / 3.;
        let extent = page_layout.extent() * scale;
        egui::ScrollArea::both().show(ui, |ui| {
            let left = ((ui.available_width() - extent.x) / 2.).max(12.);
            ui.add_space(12.);
            ui.horizontal_top(|ui| {
                ui.add_space(left);
                ui.vertical(|ui| {
                    if self.rulers {
                        let paragraph = model
                            .locate(self.session.editor.selection.0)
                            .map_or(0, |v| v.0);
                        let settings = crate::pagination::sections(&model);
                        let width = f32::from(settings[paragraph].width_points) * scale;
                        ui.horizontal(|ui| {
                            ui.add_space((extent.x - width) / 2.);
                            self.ruler(ui, width, self.zoom / 100.);
                        });
                        ui.add_space(5.);
                    }
                    ui.allocate_ui_with_layout(extent, Layout::top_down(Align::Min), |ui| {
                        let origin = ui.cursor().min;
                        let origins = page_layout.page_origins();
                        let max_width = page_layout.extent().x;
                        for (i, page) in page_layout.plan.pages.iter().enumerate() {
                            let x = (max_width - f32::from(page.settings.width_points)) / 2.;
                            page_layout.paint_background(
                                ui.ctx(),
                                ui.painter(),
                                i,
                                origin + vec2(x, origins[i]) * scale,
                                scale,
                            );
                        }
                        self.text_surface(
                            ui,
                            &model,
                            self.session.editor.typing_style,
                            self.session.editor.selection.0,
                            extent.x,
                            11. * scale,
                            self.zoom / 100.,
                            true,
                        );
                        self.record("paged-document", Rect::from_min_size(origin, extent));
                        if self.rulers {
                            self.vertical_ruler(
                                ui,
                                Rect::from_min_size(
                                    origin,
                                    vec2(
                                        extent.x,
                                        f32::from(page_layout.plan.pages[0].settings.height_points)
                                            * scale,
                                    ),
                                ),
                                self.zoom / 100.,
                            );
                        }
                    });
                });
            });
        });
    }
}
