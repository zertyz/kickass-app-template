# Production-ready seed & template for Rust Backend & Long Runner Applications

This repository presents a seed for creating backend Rust applications, with bundled libs, UIs, services & patterns.

Most likely you'll need only a subset of the features provided by this template:
   1. Services:
      - [X] Web, through `Rocket` -- with embedded static files, including an Angular app
      - [X] Telegram, through `Teloxide` -- stateless & stateful examples
         - [ ] standalone message sending
      - [X] Hooks for your own Service
         - [X] async with `Tokio`
   2. UIs:
      - [X] Console -- logging through `slog` with stdout and file sink options
      - [X] Terminal --  `tui` + `crossterm`;
      - [X] GUI -- `egui`
         - [X] `lottie` animations with `rlottie`
         - [ ] graphics with `plotters`
      - [X] Angular + Angular Universal, embedding all static files into the executable
         - [X] Angular Universal for blazing fast loading speeds of ~50ms
         - [X] Google Material theming
         - [X] example app to test the provided web services in `src/frontend/web/api`
   3. Configs:
      - [X] Application-wide `config pattern` tying all features together + customizable to include your business logic
      - [X] Command Line parsing through `structopt`, merging with the application-wide configs
      - [X] Persistent config file using `ron`

# How to use it

   * Click `Use this template`, at the top right of this github page;
   * Edit `Cargo.toml` and remove the dependencies you don't need + associated code failing to compile;
   * Remodel `src/config` module to your needs
   * Add your business logic modules
   * Inspect & update the `src/frontend` module

# Screenshots

## Angular

![rust+angular+material 1.png](screenshots/rust+angular+material%201.png)
![rust+angular+material 2.png](screenshots/rust+angular+material%202.png)
(only 44ms needed to show the content -- 13ms to load index.html + 31ms to render it. After being presented, Angular is loaded and after 664ms we have a fully working website)

![rust+angular+Pingdom+results.png](screenshots/rust+angular+Pingdom+results.png)