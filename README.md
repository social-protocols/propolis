# <img src="logo.svg" width="24" /> Propolis

*This is an archived early stage project and has been superseded by
https://github.com/social-protocols/social-network. The reasoning was: coming up
with (and answering) yes-no questions is more effort and a higher entry barrier
for participation than just posting anything and having up/downvotes - like in a
social network. Requiring this formalization of all content on a platform
creates an entry barrier, e.g. people need to formulate what they want to post
as a yes-no question. At the same time, it disallows content, which does not fit
the yes-no question model.

Our big insight was: We can drastically simplify the user interaction and allow
arbitrary content, but keep the collective intelligence aspect. That's achieved
by introducing a concept similar to twitter community notes, but in a recursive
way: Every reply to a post can become a note. And replies can have more replies,
which in turn can act as notes for the reply. Notes are AB-tested, when shown
below a post, if they change the voting behavior on the post. If a reply changes
the voting behavior, it must have added some information, which voters were not
aware of before, like a good argument.

For more details, see the global brain algorithm:
https://social-protocols.org/global-brain/*

Enable useful discussions among thousands of people.

The idea is to create a discussion interface which resembles the mechanics of a
real-world one-to-one discussion. But instead of having a single person as a
counterpart, a user has a large crowd of people on the other end. By offering
familiar actions, like _ask a yes-no question_, _answer a yes-no question_,
_clarify definitions and/or context_, users can apply strategies and experience
they know from real-world discussions. They don't have to learn a new paradignm
to engage and contribute in a discussion.

Try it: <https://propolis.fly.dev>

## Development

```bash
just reset-db
just develop
```

Open in browser: <https://localhost:8000>

## Benchmarking

Start release web server:

```bash
cargo run --release
```

Then benchmark:

```bash
just benchmark
```
