# plotty

A simple little discord bot to manage WorldGuard plots on our Minecraft server.

## Commands

### Personal

#### `/bind`

Usage: `/bind <minecraftUserName>`

Bind your Minecraft username to your Discord account. This is necessary to perform any region commands. Also, this automatically adds you to the server whitelist. You can also re-run this command if you have changed your Minecraft username.

### Regions

#### `/region list`

Usage: `/region list`

List all your registered regions.

#### `/region create`

Usage: `/region create <pos1-x> <pos1-z> <pos2-x> <pos2-z>`

Create a new personal region with the given corner coordinates.

#### `/region redefine`

Usage: `/region redefine <pos1-x> <pos1-z> <pos2-x> <pos2-z>`

Re-define the perimeter of one of your registered regions.

#### `/region member add`

Usage: `/region member add <regionName> <minecraftUsername>`

Add a member to one of your regions.

#### `/region member remove`

Usage: `/region member remove <regionName> <minecraftUsername>`

Remove a member from one of your regions.

#### `/region delete`

Usage: `/region delete <regionName>`

Delete one of your regions.

## Project Status

🚧 WIP