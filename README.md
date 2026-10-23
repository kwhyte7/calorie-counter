Simple CLI calorie counter to track your calories in the command line

if you are lost, you can use
```
calorie-counter -h 
```

to see all commands

```
Usage: calorie-counter <COMMAND>

Commands:
  day     See how many calories you've eaten today
  add     <calories> <comment> Add new listing to the database
  prev    <days> See how many previous days of calories
  remove  <id> Remove a listing by id
  all     See everything
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

all commands are aliased by the first letter (except the all and help commands) for ease of use.

commands:
day shows you how many calories you've had for the day

add <calories number> <name of entry> 
add an entry 

prev \<days> 
see how many calories total and average you've had over the last few days 

remove \<id>
remove an entry

all
show everything
