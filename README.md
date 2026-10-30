# Mainmatter Rust Assessment

> Please take no more than ~1 hour to complete this assignment. We’ll discuss your solutions in a call in a few days. Feel free to ask questions if the assignment is unclear at any point.

## Context

This repository contains a simple file-icon lookup library as may be used by IDEs for their project tree sidebar. `get_icon_for_file` and `get_icon_for_folder` accept a filepath and return the numeric ID of a corresponding icon if one is found. Each icon ID corresponds to an svg icon in `./icons`.

## Assignment

1. Implement a web server than allows users to query for file/folder icons over the network. 
2. Write a short (<= 1 page) design/implementation note describing:
  - Design choices and tradeoffs you made (high-level architecture, web server choice, libraries you picked etc.)
  - How you would productionize this service (deployment, observability, etc.)
  
Keep your audience in mind. We’re not interested in a perfect solution but one where we see your thought-process and working style in action.
  
You may add these notes at the end of this file if you wish.

## Thoughts on the exercise

The architecture, tradeoffs and production considerations are in [the design note](DESIGN.md).

My main aim was to keep this small. I would normally reach for Axum, but for two fixed routes it's overkill.

I spent some time questioning the filename allocation in the asset lookup and also tried a numeric index but eventually removed it.
It would make requests cheaper in principle, but adds more complexity to the startup logic and assumptions about the IDs.
The straightforward lookup is easy to follow and I can always revisit it if there is a reason to.
