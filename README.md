# Octoprint Command line client, written in Rust

![Workflow status](https://github.com/AntoineZen/octoprint-client-rs/actions/workflows/rust.yml/badge.svg)

This project is a client for [OctoPrint](https://octoprint.org/), written in Rust.
Note that there is already an [python cli client](https://pypi.org/project/octoprint-cli/) 
that has more feature that this project. I use this project as a vessel to practice Rust.
So for now, if you a looking for a client, you probably want the above client. toto 

# Features / Road-map

 - [x] Show server state.
 - [X] Upload G-Code file to Octoprint.
 - [X] Unit-tests.
 - [ ] Start/Stop/Pause prints.
 - [X] Connect / Disconnect printer.
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

# Configuration

The client needs two element as configuration:

 - Server URL
 - API key

They are stored in a TOML file at `~/.config/octoprint-client`. Bellow is an example configuration file:

    server_url = 'http://octoprint.local'
    api_key = '<api key here>'

If the configuration does not exist on the first run, the client will ask for those two configuration entry.

**Note** that the API key can be found in OctoPrint, as described [here](https://docs.octoprint.org/en/master/api/general.html).
