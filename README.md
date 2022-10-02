# Octoprint Command line client, in Rust

This project is a client for [OctopPint](https://octoprint.org/), written in Rust.
Note that there is already an [python cli client](https://pypi.org/project/octoprint-cli/) 
that has more feature that this project. I use this project as a vessel to practice Rust.
So for now, if you a looking for a client, you probably want the above client. toto 

# Features / Road-map

 - [x] Show server state.
 - [X] Upload G-Code file to Octoprint.
 - [ ] Start/Stop/Pause prints.
 - [ ] Connect / Disconnect printer.
 - [ ] Set extruder / bed temperatures.
 - [ ] List files.
 - [ ] Delete file.
 - [ ] Reboot / Shutdown host


# Install

Using cargo, directly from GitHub:

    $ cargo install --git https://github.com/AntoineZen/octoprint-client-rs.git

# Usage

## Get server status

Simply invoke without arguments:

    $ octoprint-client
    Connected to Octoprint version 1.7.3
    State    : Printing
    Progress : 5.6% , ends in 33 minutes
    File     : test_upload/test.gcode
    Extruder : 240°C / 240°C
    Bed      : 90°C / 90°C

## File upload

Use the `upload` subcommand:

    $ octoprint-client upload some-file.gcode
    Connected to Octoprint version 1.7.3
    Uploading "some-file.gcode"