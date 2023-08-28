**Not Production ready, still in development**


# mdSilo-spc

Self-hosted online writing platform for individual or small club, which comes as a single executable with feed subscription, publishing writing and live collaboration. Focus on the Markdown content with support on syntax highlighting, WikiLink, Hashtag, mermaid diagram, LaTex and more, be it a blog, a knowledge base, a forum or a combination of them. 

Licensed under AGPLv3.

## Features

- Easy to deploy with a single executable OR plus web app, no database to configure;    
- Support Markdown and extensions: mermaid Diagram, Table, LaTex, syntax highlighting... 
- Hashtag to organize writing;
- WikiLink to network writing;    
- Dark and Light theme;  
- Feed reader, support RSS and Atom;
- Efficient live collaboration;
- Configurable: customized css/js to style or add features... 

## Deploy 

### via released zip

**Alert! Test on Ubuntu 22.04 only**

- Download the zip from [release](https://github.com/danloh/mdSilo-spc/releases)
- Unzip, you get a `spc` executable file and a `dist` folder for collaborative editor frontend
- create a [config.toml](https://github.com/danloh/mdSilo-spc/blob/main/config.toml) file and/or run `./spc` 
- config your web server like nginx
 
## Road map 

### Subscription 
  - [X] Feed aggregator, support RSS and Atom
  - [X] Feed reader, support RSS and Atom

### Publishing
  - Writing with Markdown 
    - [X] Common Markdown 
    - [X] Highlight code block  
    - [X] Math: inline `$\LaTeX$` and block `$$\LaTeX$$` 
    - [X] Wikilink: `[[]]` 
    - [X] Diagram: mermaid... 

  - Organize writing
    - [X] Hashtag
    - [ ] viz graph
    - [ ] Storify 
  
  - Spread writing
    - [ ] RSS output

### Collaboration
  - [X] Live collaboration 
  - [X] Preview markdown and ABC Music notes 
  - [ ] Auth on collaboration
  - [ ] Live Chat on collaboration  
  - [ ] forum

## Tech Stack

- Axum(Rust): Server side rendering and API
- Askama: Template engine 
- sled: store some temp data  
- sqlite: persist data  
- WebAssembly: collaborative operation 
- React + monaco-editor: frontend of collaborative text editor 

## Credits

The collaborative editor is highly inspired by [rustpad](https://rustpad.io). 
