//! Main window: header bar with a tab bar, one `AdwTabPage` per tab, and a
//! recursive `PaneTree` per tab so the view can be split into several
//! terminals side by side. The color theme is global (Settings page) and
//! applied to every terminal pane.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::prelude::*;
use gtk4::{gio, glib};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::config::Config;
use crate::pane::TerminalPane;
use crate::theme::Theme;
use crate::tree::PaneTree;

/// One tab: the AdwTabPage plus the (fixed) holder widget that contains the
/// current root widget of the pane tree.
struct TabEntry {
    page: adw::TabPage,
    holder: gtk::Box,
    tree: Rc<RefCell<Option<PaneTree>>>,
}

pub struct AppWindow {
    pub window: adw::ApplicationWindow,
    tab_view: adw::TabView,
    config: Rc<RefCell<Config>>,
    tabs: RefCell<Vec<TabEntry>>,
    /// Every living terminal pane (across all tabs), used to apply the
    /// global theme to all of them at once.
    all_panes: RefCell<Vec<Rc<RefCell<TerminalPane>>>>,
    next_pane_id: Cell<u64>,
    tab_counter: Cell<u32>,
    default_profile: Cell<usize>,
}

impl AppWindow {
    pub fn new(app: &adw::Application, config: Rc<RefCell<Config>>) -> Rc<Self> {
        let window = adw::ApplicationWindow::new(app);
        window.set_title(Some("TermLaps"));
        window.set_default_size(960, 640);
        window.set_icon_name(Some("com.github.termlaps.Terminal"));

        let tab_view = adw::TabView::new();
        tab_view.set_hexpand(true);
        tab_view.set_vexpand(true);

        // --- Header bar: [menu] [tab bar] [split] [+] ------------------------
        let header = adw::HeaderBar::new();
        let tab_bar = adw::TabBar::new();
        tab_bar.set_view(Some(&tab_view));
        header.set_title_widget(Some(&tab_bar));

        let menu = gio::Menu::new();
        menu.append(Some("New tab"), Some("win.new-tab"));
        menu.append(Some("Split pane"), Some("win.split"));
        menu.append(Some("Close pane"), Some("win.close-pane"));
        menu.append(Some("Settings"), Some("win.settings"));
        let profiles_section = gio::Menu::new();
        for (i, profile) in config.borrow().profiles.iter().enumerate() {
            profiles_section.append(
                Some(&format!("New tab: {}", profile.name)),
                Some(&format!("win.open-profile::{i}")),
            );
        }
        menu.append_section(Some("Profiles"), &profiles_section);
        let menu_button = gtk::MenuButton::new();
        menu_button.set_icon_name("open-menu-symbolic");
        menu_button.set_menu_model(Some(&menu));
        header.pack_start(&menu_button);

        let this = Rc::new(AppWindow {
            window,
            tab_view: tab_view.clone(),
            config,
            tabs: RefCell::new(Vec::new()),
            all_panes: RefCell::new(Vec::new()),
            next_pane_id: Cell::new(1),
            tab_counter: Cell::new(0),
            default_profile: Cell::new(0),
        });

        let split_button = gtk::Button::with_label("Split");
        split_button.set_tooltip_text(Some("Split the focused pane (Ctrl+Shift+S)"));
        {
            let this = this.clone();
            split_button.connect_clicked(move |_| this.split_focused());
        }
        header.pack_end(&split_button);

        let new_tab_button = gtk::Button::with_label("+");
        new_tab_button.set_tooltip_text(Some("New tab (Ctrl+Shift+T)"));
        {
            let this = this.clone();
            new_tab_button.connect_clicked(move |_| {
                let idx = this.default_profile.get();
                this.open_tab(idx);
            });
        }
        header.pack_end(&new_tab_button);

        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(&tab_view));
        this.window.set_content(Some(&toolbar));

        this.setup_actions();

        // Window title follows the selected tab.
        {
            let this = this.clone();
            let tab_view = tab_view.clone();
            tab_view.clone().connect_notify_local(Some("selected-page"), move |_, _| {
                let page = tab_view.selected_page();
                this.update_title(page.as_ref());
            });
        }

        // Clean up our tab list when a page is closed (via the tab bar, ...).
        {
            let this = this.clone();
            tab_view.connect_close_page(move |_, page| {
                this.tabs.borrow_mut().retain(|t| t.page != *page);
                glib::Propagation::Proceed
            });
        }

        this.open_tab(0);
        this
    }

    fn setup_actions(self: &Rc<Self>) {
        let new_tab = gio::SimpleAction::new("new-tab", None);
        {
            let this = self.clone();
            new_tab.connect_activate(move |_, _| {
                let idx = this.default_profile.get();
                this.open_tab(idx);
            });
        }
        self.window.add_action(&new_tab);

        let open_profile =
            gio::SimpleAction::new("open-profile", Some(&i32::static_variant_type()));
        {
            let this = self.clone();
            open_profile.connect_activate(move |_, param| {
                if let Some(idx) = param.and_then(|p| p.get::<i32>()) {
                    this.open_tab(idx as usize);
                }
            });
        }
        self.window.add_action(&open_profile);

        let split = gio::SimpleAction::new("split", None);
        {
            let this = self.clone();
            split.connect_activate(move |_, _| this.split_focused());
        }
        self.window.add_action(&split);

        let close_pane = gio::SimpleAction::new("close-pane", None);
        {
            let this = self.clone();
            close_pane.connect_activate(move |_, _| this.close_focused_pane());
        }
        self.window.add_action(&close_pane);

        let settings = gio::SimpleAction::new("settings", None);
        {
            let this = self.clone();
            settings.connect_activate(move |_, _| this.open_settings());
        }
        self.window.add_action(&settings);

        let next_tab = gio::SimpleAction::new("next-tab", None);
        {
            let tab_view = self.tab_view.clone();
            next_tab.connect_activate(move |_, _| {
                tab_view.select_next_page();
            });
        }
        self.window.add_action(&next_tab);

        let prev_tab = gio::SimpleAction::new("prev-tab", None);
        {
            let tab_view = self.tab_view.clone();
            prev_tab.connect_activate(move |_, _| {
                tab_view.select_previous_page();
            });
        }
        self.window.add_action(&prev_tab);
    }

    fn new_pane_id(&self) -> u64 {
        let id = self.next_pane_id.get();
        self.next_pane_id.set(id + 1);
        id
    }

    /// Apply the global color theme to every terminal and save the config so
    /// it persists across restarts.
    pub fn apply_theme(self: &Rc<Self>, theme: Theme) {
        for pane in self.all_panes.borrow().iter() {
            pane.borrow().set_theme(theme);
        }
        let mut config = self.config.borrow_mut();
        config.settings.theme = theme;
        if let Err(err) = config.save() {
            eprintln!("termlaps: could not save the configuration: {err}");
        }
    }

    /// Settings page: lets the user pick the app-wide color theme.
    pub fn open_settings(self: &Rc<Self>) {
        let dialog = adw::PreferencesDialog::new();
        dialog.set_title("Settings");

        let page = adw::PreferencesPage::new();
        let group = adw::PreferencesGroup::builder()
            .title("Appearance")
            .build();

        let model = gtk::StringList::new(&Theme::ALL.map(|t| t.label()));
        let current = self.config.borrow().settings.theme.index();
        let row = adw::ComboRow::builder()
            .title("Color theme")
            .subtitle("Applied to all terminals, saved across restarts")
            .model(&model)
            .selected(current)
            .build();
        {
            let this = self.clone();
            row.connect_notify_local(Some("selected"), move |row, _| {
                this.apply_theme(Theme::from_index(row.selected()));
            });
        }

        group.add(&row);
        page.add(&group);
        dialog.add(&page);
        dialog.present(Some(&self.window));
    }

    /// Open a new tab running the given profile. Tabs are named
    /// "Terminal 1", "Terminal 2", ...
    pub fn open_tab(self: &Rc<Self>, profile_idx: usize) {
        let profile = {
            let config = self.config.borrow();
            config
                .profiles
                .get(profile_idx)
                .or_else(|| config.profiles.first())
                .cloned()
        };
        let Some(profile) = profile else {
            eprintln!("termlaps: no profile defined in the configuration");
            return;
        };
        self.default_profile.set(profile_idx);

        let tab_number = {
            let n = self.tab_counter.get() + 1;
            self.tab_counter.set(n);
            n
        };

        let on_exit: Rc<dyn Fn(u64)> = {
            let this = self.clone();
            Rc::new(move |id| this.on_child_exited(id))
        };
        let pane = TerminalPane::new(
            self.new_pane_id(),
            profile.clone(),
            &self.config.borrow().settings,
            on_exit,
        );
        self.all_panes.borrow_mut().push(pane.clone());
        let tree: Rc<RefCell<Option<PaneTree>>> =
            Rc::new(RefCell::new(Some(PaneTree::leaf(pane.clone()))));

        let holder = gtk::Box::new(gtk::Orientation::Vertical, 0);
        holder.set_hexpand(true);
        holder.set_vexpand(true);
        holder.append(&tree.borrow().as_ref().map(PaneTree::widget).unwrap());

        let page = self.tab_view.append(&holder);
        page.set_title(&format!("Terminal {tab_number}"));
        page.set_tooltip(&format!("{} — {}", profile.name, profile.command.join(" ")));

        self.tabs.borrow_mut().push(TabEntry {
            page: page.clone(),
            holder,
            tree,
        });
        self.tab_view.set_selected_page(&page);
        pane.borrow().term.grab_focus();
        self.update_title(Some(&page));
    }

    fn current_entry(&self) -> Option<(adw::TabPage, gtk::Box, Rc<RefCell<Option<PaneTree>>>)> {
        let page = self.tab_view.selected_page()?;
        self.tabs
            .borrow()
            .iter()
            .find(|t| t.page == page)
            .map(|t| (t.page.clone(), t.holder.clone(), t.tree.clone()))
    }

    fn update_title(&self, page: Option<&adw::TabPage>) {
        match page {
            Some(page) => self
                .window
                .set_title(Some(&format!("TermLaps — {}", page.title()))),
            None => self.window.set_title(Some("TermLaps")),
        }
    }

    /// Detach the holder's current children, run `f` on the tree and re-attach
    /// the resulting root widget. Returns false if the tree became empty.
    fn reattach(
        holder: &gtk::Box,
        tree: &Rc<RefCell<Option<PaneTree>>>,
        f: impl FnOnce(PaneTree) -> Option<PaneTree>,
    ) -> bool {
        while let Some(child) = holder.first_child() {
            holder.remove(&child);
        }
        let new_tree = tree.borrow_mut().take().and_then(f);
        match new_tree {
            Some(new_tree) => {
                let widget = new_tree.widget();
                if widget.parent().is_some() {
                    widget.unparent();
                }
                *tree.borrow_mut() = Some(new_tree);
                holder.append(&widget);
                true
            }
            None => false,
        }
    }

    /// Split the focused pane of the current tab side by side, reusing the
    /// profile of the pane being split.
    pub fn split_focused(self: &Rc<Self>) {
        let Some((_, holder, tree)) = self.current_entry() else {
            return;
        };
        let (target, profile) = {
            let t = tree.borrow();
            let Some(t) = t.as_ref() else { return };
            let id = t.focused_id().unwrap_or_else(|| t.first_id());
            match t.profile_of(id) {
                Some(p) => (id, p),
                None => return,
            }
        };

        let on_exit: Rc<dyn Fn(u64)> = {
            let this = self.clone();
            Rc::new(move |id| this.on_child_exited(id))
        };
        let pane = TerminalPane::new(
            self.new_pane_id(),
            profile,
            &self.config.borrow().settings,
            on_exit,
        );

        // Detach the old root first so the new paned can adopt it cleanly.
        while let Some(child) = holder.first_child() {
            holder.remove(&child);
        }
        let widget = {
            let mut t = tree.borrow_mut();
            let Some(tree_ref) = t.as_mut() else { return };
            if !tree_ref.split(target, pane.clone()) {
                return;
            }
            tree_ref.widget()
        };
        if widget.parent().is_some() {
            widget.unparent();
        }
        holder.append(&widget);
        // Give both panes half of the width on the first split.
        if let Some(paned) = widget.downcast_ref::<gtk::Paned>() {
            let width = holder.width();
            if width > 0 {
                paned.set_position(width / 2);
            }
        }
        self.all_panes.borrow_mut().push(pane.clone());
        pane.borrow().term.grab_focus();
    }

    /// Close the focused pane of the current tab; closes the tab itself when
    /// it was the last pane.
    pub fn close_focused_pane(self: &Rc<Self>) {
        let Some((page, holder, tree)) = self.current_entry() else {
            return;
        };
        let target = {
            let t = tree.borrow();
            t.as_ref().map(|t| t.focused_id().unwrap_or_else(|| t.first_id()))
        };
        let Some(target) = target else { return };

        if !Self::reattach(&holder, &tree, |t| t.remove(target)) {
            self.tab_view.close_page(&page);
        }
        self.all_panes.borrow_mut().retain(|p| p.borrow().id != target);
    }

    /// A pane's child process exited: remove that pane (or close its tab).
    fn on_child_exited(self: &Rc<Self>, id: u64) {
        let entry = self
            .tabs
            .borrow()
            .iter()
            .find(|t| {
                t.tree
                    .borrow()
                    .as_ref()
                    .map(|tree| tree.contains(id))
                    .unwrap_or(false)
            })
            .map(|t| (t.page.clone(), t.holder.clone(), t.tree.clone()));
        let Some((page, holder, tree)) = entry else {
            return;
        };

        if !Self::reattach(&holder, &tree, |t| t.remove(id)) {
            self.tab_view.close_page(&page);
        }
        self.all_panes.borrow_mut().retain(|p| p.borrow().id != id);
    }
}
