use iui::controls::{
    Button, Combobox, Entry, HorizontalBox, Label, RadioButtons, Spacer, TextEntry, VerticalBox, HorizontalSeparator,
};
use iui::prelude::*;
use nfd::Response;
use std::fs::OpenOptions;
use std::collections::HashMap;
use std::panic;
use std::path::{Path, PathBuf};
use std::process;
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use which::which;

mod updater;

fn mono_error() {
    let ui = UI::init().expect("Couldn't create UI!");

    let mut win = Window::new(&ui, "EverInst", 200, 200, WindowType::NoMenubar);

    let mut container = VerticalBox::new(&ui);
    let spacer = Spacer::new(&ui);

    let mut go_there = Button::new(&ui, "Go there");
    go_there.on_clicked(&ui, |_| {
        open::that("https://www.mono-project.com/download/stable/").unwrap();
    });

    let mut exit = Button::new(&ui, "Exit");
    exit.on_clicked(&ui, {
        let ui = ui.clone();
        move |_| {
            ui.quit();
        }
    });

    let label = Label::new(&ui, "You must have Mono installed to use EverInst.\nGet it from https://www.mono-project.com/download/stable/.");

    container.append(&ui, label, LayoutStrategy::Compact);
    container.append(&ui, spacer, LayoutStrategy::Stretchy);
    container.append(&ui, go_there, LayoutStrategy::Compact);
    container.append(&ui, exit, LayoutStrategy::Compact);

    container.set_padded(&ui, true);

    win.set_child(&ui, container);
    win.show(&ui);
    ui.main();
}

fn register_uri_handler() {
    let exe = std::env::current_exe().unwrap().to_str().unwrap().to_string();
    let app = system_uri::App::new(
        "space.leo60228.everinst".to_string(),
        "leo60228".to_string(),
        "EverInst".to_string(),
        exe,
        None
    );
    if let Err(err) = system_uri::install(&app, &["everest".to_string()]) {
        println!("WARNING: Failed to add URI handler! Error: {}", err);
    }
}

fn handle_everest_uri() {
    // STATE
    let steam = find_steam();

    let ui = UI::init().expect("Couldn't create UI!");
    let mut win = Window::new(&ui, "EverInst", 200, 300, WindowType::NoMenubar);

    let (tx, rx) = mpsc::channel();
    let (ready_tx, ready_rx) = mpsc::channel();

    // PAGES
    let mut select = VerticalBox::new(&ui);
    let mut install = VerticalBox::new(&ui);
    let mut finish = VerticalBox::new(&ui);

    // CELESTE PAGE

    let mut next_button = Button::new(&ui, "Next");

    let mut radio = RadioButtons::new(&ui);
    let mut file_entry = Entry::new(&ui);
    let mut file_button = Button::new(&ui, "...");

    if steam.is_some() {
        radio.append(&ui, "Steam");
    }

    radio.append(&ui, "Local path:");

    if steam.is_some() && radio.selected(&ui) == 0 {
        file_entry.set_value(&ui, steam.as_ref().unwrap().to_str().unwrap());
    }

    let mut file_chooser = HorizontalBox::new(&ui);

    file_chooser.set_padded(&ui, true);

    file_button.on_clicked(&ui, |_| {
        if let Ok(Response::Okay(file)) = nfd::open_file_dialog(Some("exe"), None) {
            file_entry.set_value(&ui, &file);

            if steam.is_some() {
                radio.set_selected(&ui, 1);
            }

            if Path::new(&file).is_file() {
                next_button.enable(&ui);
            } else {
                next_button.disable(&ui);
            }
        }
    });

    radio.on_selected(&ui, |btn| {
        if steam.is_some() && btn != 1 {
            file_entry.set_value(&ui, steam.as_ref().unwrap().to_str().unwrap());
            next_button.enable(&ui);
        } else if steam.is_some() {
            file_entry.set_value(&ui, "");
        }

        if Path::new(&file_entry.value(&ui)).is_file() {
            next_button.enable(&ui);
        } else {
            next_button.disable(&ui);
        }
    });

    file_entry.on_changed(&ui, |path| {
        if steam.is_some() {
            radio.set_selected(&ui, 1);
        }

        if Path::new(&path).is_file() {
            next_button.enable(&ui);
        } else {
            next_button.disable(&ui);
        }
    });

    file_chooser.append(&ui, file_entry.clone(), LayoutStrategy::Stretchy);
    file_chooser.append(&ui, file_button.clone(), LayoutStrategy::Compact);

    next_button.on_clicked(&ui, |_| {
        win.set_child(&ui, install.clone());

        let path = Path::new(&file_entry.value(&ui))
            .parent()
            .unwrap()
            .to_path_buf();

        ready_tx
            .send(path)
            .unwrap();
    });

    select.append(
        &ui,
        Label::new(&ui, "Select Celeste location:"),
        LayoutStrategy::Compact,
    );
    select.append(&ui, radio.clone(), LayoutStrategy::Compact);
    select.append(&ui, file_chooser, LayoutStrategy::Compact);
    select.append(&ui, Spacer::new(&ui), LayoutStrategy::Stretchy);
    select.append(&ui, next_button, LayoutStrategy::Compact);

    select.set_padded(&ui, true);

    // INSTALL PAGE (background thread)
    thread::spawn(move || {
        let url = std::env::args().nth(1).unwrap();
        let url = (&url["everest:".len()..]).split(",").next().unwrap();

        let mut path = ready_rx.recv().unwrap();
        path.push("Mods/");

        tx.send("downloading".to_string()).unwrap();

        println!("{}", url);

        let mut resp = reqwest::get(url).unwrap();

        assert!(resp.status().is_success());

        path.push(resp.url().path_segments().unwrap().last().unwrap());
        let mut file = OpenOptions::new().write(true).create(true).open(&path).unwrap();
        resp.copy_to(&mut file).unwrap();

        tx.send("done".to_string()).unwrap();
    });

    install.set_padded(&ui, true);

    // FINISH PAGE
    let mut exit = Button::new(&ui, "Exit");
    exit.on_clicked(&ui, {
        let ui = ui.clone();
        move |_| {
            ui.quit();
        }
    });

    let mut label_holder = HorizontalBox::new(&ui);
    label_holder.append(&ui, Spacer::new(&ui), LayoutStrategy::Stretchy);
    label_holder.append(
        &ui,
        Label::new(&ui, "Your mod has finished installing!"),
        LayoutStrategy::Compact,
    );
    label_holder.append(&ui, Spacer::new(&ui), LayoutStrategy::Stretchy);

    finish.append(&ui, Spacer::new(&ui), LayoutStrategy::Stretchy);
    finish.append(&ui, label_holder, LayoutStrategy::Compact);
    finish.append(&ui, Spacer::new(&ui), LayoutStrategy::Stretchy);
    finish.append(&ui, exit, LayoutStrategy::Compact);

    finish.set_padded(&ui, true);

    // DISPLAY WINDOW
    win.set_child(&ui, select);
    win.show(&ui);

    let mut eloop = ui.event_loop();
    eloop.on_tick(&ui, || {
        // INSTALL PAGE (event loop)
        if let Ok(msg) = rx.try_recv() {
            if msg == "downloading" {
                install.append(
                    &ui,
                    Label::new(&ui, "Downloading..."),
                    LayoutStrategy::Compact,
                );
            } else if msg == "done" {
                win.set_child(&ui, finish.clone());
            }
        }
    });
    eloop.run_delay(&ui, 200);
}

fn main() {
    // immediately exit on panic
    // TODO: display graphical message before exiting (via event loop)
    let orig_handler = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        orig_handler(panic_info);
        process::exit(1);
    }));

    // use system ssl certs
    openssl_probe::init_ssl_cert_env_vars();

    register_uri_handler();

    if let Some(_) = std::env::args().nth(1) {
        handle_everest_uri();
        return;
    }

    let mut mono: Option<PathBuf> = None;

    if !cfg!(windows) {
        mono = which("mono").ok();

        if mono == None {
            mono_error();
            std::process::exit(127);
        }
    }

    display(mono);
}

fn find_steam() -> Option<PathBuf> {
    let path = if cfg!(windows) {
        Some(PathBuf::from(
            "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Celeste\\Celeste.exe",
        ))
    } else {
        let mut data = dirs::data_local_dir();

        if let Some(local) = &mut data {
            local.push("Steam/steamapps/common/Celeste/Celeste.exe");
        }

        data
    };

    if let Some(loc) = path {
        if loc.exists() {
            return Some(loc);
        }
    }

    None
}

fn display(mono: Option<PathBuf>) {
    // STATE
    let steam = find_steam();

    let ui = UI::init().expect("Couldn't create UI!");
    let mut win = Window::new(&ui, "EverInst", 200, 300, WindowType::NoMenubar);

    let mut version: Option<updater::EverestVersion> = None;

    let (tx, rx) = mpsc::channel();
    let (ready_tx, ready_rx) = mpsc::channel();

    // PAGES
    let mut select = VerticalBox::new(&ui);
    let mut install = VerticalBox::new(&ui);
    let mut finish = VerticalBox::new(&ui);

    // CELESTE PAGE

    let mut next_button = Button::new(&ui, "Next");

    let mut radio = RadioButtons::new(&ui);
    let mut file_entry = Entry::new(&ui);
    let mut file_button = Button::new(&ui, "...");

    if steam.is_some() {
        radio.append(&ui, "Steam");
    }

    radio.append(&ui, "Local path:");

    if steam.is_some() && radio.selected(&ui) == 0 {
        file_entry.set_value(&ui, steam.as_ref().unwrap().to_str().unwrap());
    }

    let mut file_chooser = HorizontalBox::new(&ui);

    file_chooser.set_padded(&ui, true);

    file_button.on_clicked(&ui, |_| {
        if let Ok(Response::Okay(file)) = nfd::open_file_dialog(Some("exe"), None) {
            file_entry.set_value(&ui, &file);

            if steam.is_some() {
                radio.set_selected(&ui, 1);
            }

            if Path::new(&file).is_file() {
                next_button.enable(&ui);
            } else {
                next_button.disable(&ui);
            }
        }
    });

    radio.on_selected(&ui, |btn| {
        if steam.is_some() && btn != 1 {
            file_entry.set_value(&ui, steam.as_ref().unwrap().to_str().unwrap());
            next_button.enable(&ui);
        } else if steam.is_some() {
            file_entry.set_value(&ui, "");
        }

        if Path::new(&file_entry.value(&ui)).is_file() {
            next_button.enable(&ui);
        } else {
            next_button.disable(&ui);
        }
    });

    file_entry.on_changed(&ui, |path| {
        if steam.is_some() {
            radio.set_selected(&ui, 1);
        }

        if Path::new(&path).is_file() {
            next_button.enable(&ui);
        } else {
            next_button.disable(&ui);
        }
    });

    file_chooser.append(&ui, file_entry.clone(), LayoutStrategy::Stretchy);
    file_chooser.append(&ui, file_button.clone(), LayoutStrategy::Compact);

    let mut selector = Combobox::new(&ui);
    let versions = updater::get_versions();
    let mut version_map: HashMap<usize, updater::EverestVersion> = HashMap::new();

    for (i, version) in versions.iter().enumerate() {
        selector.append(&ui, &format!("{} ({})", version.ver, version.branch));
        version_map.insert(i, version.clone());
    }

    selector.set_selected(&ui, 0);

    next_button.on_clicked(&ui, |_| {
        let selected = selector.selected(&ui) as usize;

        version = Some(version_map[&selected].clone());
        win.set_child(&ui, install.clone());

        let path = Path::new(&file_entry.value(&ui))
            .parent()
            .unwrap()
            .to_path_buf();

        ready_tx
            .send((version.clone().unwrap(), path, mono.clone()))
            .unwrap();
    });

    select.append(
        &ui,
        Label::new(&ui, "Select Celeste location:"),
        LayoutStrategy::Compact,
    );
    select.append(&ui, radio.clone(), LayoutStrategy::Compact);
    select.append(&ui, file_chooser, LayoutStrategy::Compact);
    select.append(&ui, HorizontalSeparator::new(&ui), LayoutStrategy::Compact);
    select.append(
        &ui,
        Label::new(&ui, "Select Everest version:"),
        LayoutStrategy::Compact,
    );
    select.append(&ui, selector.clone(), LayoutStrategy::Compact);
    select.append(&ui, Spacer::new(&ui), LayoutStrategy::Stretchy);
    select.append(&ui, next_button, LayoutStrategy::Compact);

    select.set_padded(&ui, true);

    // INSTALL PAGE (background thread)
    thread::spawn(move || {
        let (version, game, mono) = match ready_rx.recv() {
            Ok(tup) => tup,
            Err(_) => return,
        };

        tx.send("downloading".to_string()).unwrap();

        println!("{}", version.url);

        let mut file = std::io::Cursor::new(vec![]);
        let mut resp = reqwest::get(&version.url).unwrap();

        assert!(resp.status().is_success());

        resp.copy_to(&mut file).unwrap();

        tx.send("extracting".to_string()).unwrap();

        let mut zip = zip::ZipArchive::new(file).unwrap();

        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();

            let mut path = game.clone();
            path.push(&file.name()[5..]);

            println!("{:?}", path);

            if file.name().ends_with("/") {
                std::fs::create_dir_all(path).unwrap();
            } else {
                let mut disk_file = std::fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .open(path)
                    .unwrap();

                std::io::copy(&mut file, &mut disk_file).unwrap();
            }
        }

        tx.send("installing".to_string()).unwrap();

        let installer = if let Some(mono) = mono {
            Command::new(mono)
                .arg("MiniInstaller.exe")
                .current_dir(game)
                .status()
                .unwrap()
        } else {
            Command::new("MiniInstaller.exe")
                .current_dir(game)
                .status()
                .unwrap()
        };

        assert!(installer.success());

        tx.send("done".to_string()).unwrap();
    });

    install.set_padded(&ui, true);

    // FINISH PAGE
    let mut exit = Button::new(&ui, "Exit");
    exit.on_clicked(&ui, {
        let ui = ui.clone();
        move |_| {
            ui.quit();
        }
    });

    let mut label_holder = HorizontalBox::new(&ui);
    label_holder.append(&ui, Spacer::new(&ui), LayoutStrategy::Stretchy);
    label_holder.append(
        &ui,
        Label::new(&ui, "Everest has finished installing!"),
        LayoutStrategy::Compact,
    );
    label_holder.append(&ui, Spacer::new(&ui), LayoutStrategy::Stretchy);

    finish.append(&ui, Spacer::new(&ui), LayoutStrategy::Stretchy);
    finish.append(&ui, label_holder, LayoutStrategy::Compact);
    finish.append(&ui, Spacer::new(&ui), LayoutStrategy::Stretchy);
    finish.append(&ui, exit, LayoutStrategy::Compact);

    finish.set_padded(&ui, true);

    // DISPLAY WINDOW
    win.set_child(&ui, select);
    win.show(&ui);

    let mut eloop = ui.event_loop();
    eloop.on_tick(&ui, || {
        // INSTALL PAGE (event loop)
        if let Ok(msg) = rx.try_recv() {
            if msg == "downloading" {
                install.append(
                    &ui,
                    Label::new(&ui, "Downloading..."),
                    LayoutStrategy::Compact,
                );
            } else if msg == "extracting" {
                install.append(
                    &ui,
                    Label::new(&ui, "Extracting..."),
                    LayoutStrategy::Compact,
                );
            } else if msg == "installing" {
                install.append(
                    &ui,
                    Label::new(&ui, "Installing..."),
                    LayoutStrategy::Compact,
                );
            } else if msg == "done" {
                win.set_child(&ui, finish.clone());
            }
        }
    });
    eloop.run_delay(&ui, 200);
}
