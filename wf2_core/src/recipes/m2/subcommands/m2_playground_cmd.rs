use crate::commands::CliCommand;
use crate::conditions::file_present::FilePresent;
use crate::conditions::question::Question;
use crate::context::Context;
use crate::recipes::m2::subcommands::m2_playground::{
    get_composer_json, get_project_files, write_auth_json, M2Playground,
};
use crate::task::Task;
use ansi_term::Colour::{Cyan, Green, Red};
use clap::{App, Arg, ArgMatches};
use futures::future::lazy;
use std::path::Path;
use std::sync::Arc;

pub struct M2PlaygroundCmd(String);

const NAME: &'static str = "m2-playground";

impl M2PlaygroundCmd {
    pub fn new() -> M2PlaygroundCmd {
        M2PlaygroundCmd(String::from(NAME))
    }
}

impl<'a, 'b> CliCommand<'a, 'b> for M2PlaygroundCmd {
    fn name(&self) -> String {
        String::from(NAME)
    }

    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Vec<Task> {
        let cwd = ctx.cwd.clone();
        let version = matches.and_then(|m| m.value_of("version"));
        let username = matches.and_then(|m| m.value_of("username"));
        let password = matches.and_then(|m| m.value_of("password"));
        let dirname = matches.and_then(|m| m.value_of("output")).unwrap_or(NAME);
        let force = matches.map(|m| m.is_present("force")).unwrap_or(false);

        if let None = version {
            return vec![Task::notify_error("didn't get a valid version")];
        }

        let mut pg = M2Playground::from_file();
        let from_file = pg.is_some();
        let target_file = M2Playground::output_file();
        let version = version.expect("guarded above");

        let username = username
            .or(pg.as_ref().map(|x| x.username.as_str()))
            .expect("guarded");
        let password = password
            .or(pg.as_ref().map(|x| x.password.as_str()))
            .expect("guarded");

        let target_dir = cwd.join(dirname);

        let pg = M2Playground {
            version: String::from(version),
            dir: target_dir.clone(),
            username: String::from(username),
            password: String::from(password),
        };

        let pg = Arc::new(pg);
        let pg_1 = pg.clone();
        let pg_2 = pg.clone();
        let pg_3 = pg.clone();

        let get_files = Task::Exec {
            description: Some(format!("Get M2 project files")),
            exec: Box::new(lazy(move || get_project_files(&pg_1))),
        };

        let get_composer_json = Task::Exec {
            description: Some(format!("Get M2 composer.json file ")),
            exec: Box::new(lazy(move || get_composer_json(&pg_2))),
        };

        let auth_json = Task::Exec {
            description: Some(format!("Write auth.json")),
            exec: Box::new(lazy(move || write_auth_json(&pg_3))),
        };

        let save_creds = if !from_file {
            Task::conditional(
                vec![Box::new(Question::new(format!(
                    "{}: Save username/password for next time?",
                    Green.paint("[wf2 info]")
                )))],
                vec![Task::file_write(
                    target_file.expect("target file"),
                    "Writes the credentials to file for next time",
                    serde_json::to_vec_pretty(&*pg.clone()).expect("serde=safe"),
                )],
                vec![],
                Some(String::from("Save creds for next time")),
            )
        } else {
            Task::Noop
        };

        let save_cred_iter = vec![save_creds].into_iter();

        let base_tasks = vec![
            Task::notify_info(format!(
                "Getting the Magento 2 project files for version `{}` (this can take a while)",
                Cyan.paint(pg.version.clone())
            )),
            get_files,
            Task::notify_info(format!(
                "Getting the correct `{}` file",
                Cyan.paint("composer.json")
            )),
            get_composer_json,
            Task::notify_info(format!("Creating an `{}` file", Cyan.paint("auth.json"))),
            auth_json,
            Task::notify_info(format!("{}", Green.paint("All done :)"))),
        ];

        // If -f was given just add a verification step to ensure it was intended
        if force {
            let wipe = Task::dir_remove(target_dir.clone(), "Remove an existing folder");
            let warning = format!(
                "{}: `{}` will be {} - are you {} sure about this?",
                Green.paint("[wf2 info]"),
                target_dir.clone().display(),
                Red.paint("deleted"),
                Cyan.paint("REALLY")
            );
            return vec![Task::conditional(
                vec![Box::new(Question::new(warning))],
                vec![Task::notify_info("Deleting previous directory"), wipe]
                    .into_iter()
                    .chain(base_tasks.into_iter())
                    .chain(save_cred_iter)
                    .collect::<Vec<Task>>(),
                vec![Task::notify_info("Aborted... phew")],
                Some("Verify that the folder should be deleted"),
            )];
        }

        // if we get here, it's the 'safe' version where we wouldn't override
        // an existing directory
        vec![Task::conditional(
            vec![Box::new(FilePresent::new(target_dir.clone(), true))],
            base_tasks
                .into_iter()
                .chain(save_cred_iter)
                .collect::<Vec<Task>>(),
            vec![Task::notify_error(format!(
                "Cannot overwrite existing directory (use -f to override) `{}`",
                target_dir.clone().display()
            ))],
            Some("Verify the folder is absent before downloading anything"),
        )]
    }

    fn subcommands(&self) -> Vec<App<'a, 'b>> {
        let pg_file = M2Playground::from_file();
        let args_required = pg_file.is_none();
        vec![App::new(NAME)
            .about("Create a fresh install of M2")
            .arg_from_usage("<version> 'Which magento version to use'")
            .after_help("Example: wf2 playground 2.3.3")
            .arg(
                Arg::with_name("username")
                    .long("username")
                    .takes_value(true)
                    .required(args_required)
                    .help("magento username"),
            )
            .arg(
                Arg::with_name("password")
                    .long("password")
                    .takes_value(true)
                    .required(args_required)
                    .help("magento password"),
            )
            .arg_from_usage("-f --force 'wipe an existing folder before starting'")
            .arg_from_usage("-o --output [dirname] 'name of the directory to create'")]
    }
}
