# too-much-spare-time-bot
Discord bot to count how many messages a user has sent and assign roles based on that count

## Installation 
Requirments:
- Cargo
- MySQL database

`cargo install --git https://github.com/TheDutchMC/too-much-spare-time-bot.git`

## Usage
`too-much-spare-time --config <path to config file>`

## Configuration 
```yaml
discord:
  token: "YOUR TOKEN HERE"
mysql:
  host: "localhost"
  database: "too_much_spare_time_bot"
  username: "too_much_spare_time_bot"
  password: "123"
roles:
  - id: role_id_here
    messages: 15
```

## License
This project is dual licensed under the MIT and Apache-2.0 license, at your discretion