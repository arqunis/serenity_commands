# Serenity commands

A utility library for easily defining and parsing interactions,
particularly application commands, in [Serenity][serenity].

An interaction is, [as Discord puts it][interactions], "\[a\] message
that your application receives when a user uses an application command
or a message component." This library leans towards application
commands, which are interfaces between a user of a bot and the Discord
client for invoking behaviour of a bot.

## Application commands

Application commands are a standard and official method of defining
user-invocable actions that are directly integrated with the Discord
client.

There are three types of application commands:

1. Slash commands
2. User commands
3. Message commands

**Slash commands** are commands invoked with the `/` prefix. They are
your typical notion of text-based commands you would implement
yourself by parsing the content of a message, but they are known to
the Discord client in advance, which allows for perks like
autocompletion, and for argument checks (checks for argument count,
type-checking), and permission checks to be performed before the
command is sent to the bot to process.

User and message commands are UI-based commands that are invoked by
pressing a button in a context menu, which is shown by right-clicking
a user or message, respectively. Because they appear in a context
menu, they may also be referred to as **context-menu commands**,

The library is specifically optimised towards slash commands, as they
will typically comprise the majority of application commands in a bot.

## Why use this library rather than Serenity directly

The [Serenity][serenity] library has native support for interactions,
but they are clunky to use, as they involve a lot of builders to
define them and all of their values. Parsing them is even worse; it is
entirely your responsibility to extract values from the right places,
ensure they are correct, and in the format you want them.

To demonstrate, here is how you would define and parse a simple
`/ping` command, which accepts an `n` parameter for the amount of
pings, in Serenity:

```rust
// TODO
```

And here is how in the library:

```rust
// TODO
```

You may find full-fledged examples of the library in the
[`examples`](./examples) directory.

[serenity]: https://github.com/serenity-rs/serenity
[interactions]: https://discord.com/developers/docs/interactions/receiving-and-responding#interactions
