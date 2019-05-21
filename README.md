# Geometry Dash Caching Framework

GDCF provides the means to process all sorts of Geometry Dash releated data. Originally intended to be a caching API client for the `boomlings.com` HTTP API (hence the name "caching framework"), is has since gained the means to efficiently process robtop's data formats and support for reading/modifying the game's application data is planned.

In general, GDCF itself are the following 3 crates:

## `gdcf_model`

This crate contains `structs` modelling the various objects you can read from the game's files or receive from the servers. It optionally provides [serde](https://github.com/serde-rs/serde) support, if you wanna get a sane representation of the stuff.

## `gdcf_parse`

This crate contains efficient parsers for RobTop's data structures and the means to create custom ones yourself (via the `parser!` macro). All parsers in this crate use no allocations until it actually comes to constructing the `gdcf_model` structs (though we could just slap a lifetime on them and use `&str` instead of `String`).

A benchmark with `criterion.rs` has showed, that `gdcf_parse` can calculate the level length of bloodlust in just `~57ms` (Calculating the level length requires parsing all objects, extracting the speed portals, sorting them, and doing some simple maths)!

## `gdcf`

This crate is, although the smallest, the actual heart of the project. It defines traits for how a API client to retrieve, and a cache to store, objects from `gdcf_model` should look. It then defines the `Gdcf` struct, which allows you to make requests through the API client, where the responses are stored in the cache. For each request, it first looks into the cache, to see if the request _could_ be satisfied using cached data. There are three possible outcomes here:

- _The requested data isn't cached_: In this case, GDCF makes a request using the provided API client and returns a future. That future first awaits the API request's completion, stores the response in the provided cache (for later use), and then resolves to the received data
- _The requested data is cached, but the cached value is considered outdated_: In this case, GDCF does the same as above, but returns the cached value along with the future
- _The requested data is cached and valid_: In this case, GDCF never makes any API request and simply returns the cached data

### Why you would want to do this

There are multiple reasons why this makes sense:

- The Geometry Dash servers are notoriously slow, with response times of multiple seconds. In environments, where it's simply not acceptable to wait that long, but it's okay to sometimes produce no data at all (e.g. a website that wants to embed information about levels, but want to keep its own request times down, like say, pointercrate), you can always use the cached value (which is only a database query away, if it exists), and schedule the provided future on some background worker.
- The Geometry Dash servers are unreliable. They error out randomly, they provide incomplete HTTP responses and they are down very often. While the first two problems aren't really this crates responsibility (it doesn't define how the API client works, just what data it should provide. The `gdrs` crate below however, does adress these problems), it allows you to still access the data in your cache if the servers happen to be down.
- GDCF is very good at stitching together data. Robtop's server responses are built exactly for how the Geometry Dash client works. If you download a level, you won't get song or creator information, because that data is downloaded when you browse the level list. GDCF can automatically detect the absense of relevant data and provide it either from cache, or make additionally requests to retrieve it. Want to download a level and have the creators profile as associate user data? GDCF's got you covered.

## `gdrs`

This crate is a reference implementation of an API client to use with the `gdcf` crate. It implements the serialization of requests and parses the responses with `gdcf_parse`. It implements automatic retry (with exponentiall backoff) for when the boomlings servers decide to act up.

## `gdcf_diesel`

This crate implements a postgres and an sqlite cache for use with `gdcf`, based on diesel. Generally, the code in this crate is pretty ugly, 25% of it is a single macro, which generates around 90% of the final code. It gets the job done though and is better than the old, self-rolled sql query builder.

## Planned features

- Parsing of `CCLocalLevels.dat` and maybe `CCGameManager.dat`. This would, for example, allow us to write a program that automatically fixes broken savefiles (although using GDCF for that is really overkill, as it can be done with a 20 line python script)
- Support for endpoints and other things. Right now GDCF is mainly focused on levels. It'd be nice if it supported things like leaderboards as well (and maybe even support the parts of the API that require authentication)
- And obviously figure out more about what the yet-unidentified fields (called `index_*`) represent.

## Potential use cases

- _Caching proxy servers for boomlings.com_: By replicating the endpoints of the boomlings API, one could use GDCF to write a caching proxy for the GD servers. Or, if you use a no-op API client, a private server (though this would require a lot more support of things in GDCF itself).
- _Caching API clients_: This is what I originally designed the whole thing for and how it's used on pointercrate.
- _A part of a custom Geometry Dash level editor_: If one were to write functions to reverse the work done in `gdcf_parse` and once the support for processing `CCLocalLevels.dat` is done, `gdcf_model` and `gdcf_parse` could be used as the building blocks for a custom Geometry Dash level editor.
- _Collecting statistical data about GD_: Since `gdrs` is very good at recovering from errors, one could use the built-in pagination support (which is better than the one in the official client, go figure) to clone certain sections of the Geometry Dash databases. If you write clever code, you could build working leaderboards on top of GDCF. Or find out which custom song has the most uses in 2.1 levels.

## Disclaimer

This whole thing is still in very early stages of development. The only documented parts are the structs modelling Geometry Dash objects, and the past few weeks I've rewritten the core gdcf crate at least 3 times. I've only uploaded it here because it was simpler than creating more private git repos on pointercrate itself and because some people I'm working with for an update to pointercrate need it as a reference. If you still want to use this, here's a [discord server](https://discord.gg/sQewUEB)

## Example

Here's an example of how to download pages 6 through 55 of featured demons levels using GDCF!

```rust
// First we need to configure the cache. Here we're using a sqlite in-memory database
// whose cache entries expire after 30 minutes.
let mut config = DatabaseCacheConfig::sqlite_memory_config();
config.invalidate_after(Duration::minutes(30));

// Then we can create the actual cache and API wrapper
let cache = DatabaseCache::new(config);
let client = BoomlingsClient::new();

// A database cache needs to go through initialization before it can be used, as it
// needs to create all the required tables
cache.initialize()?;

// Then we can create an instance of the Gdcf struct, which we will use to
// actually make all our requests
let gdcf = Gdcf::new(client, cache);

// And we're good to go! To make a request, we need to initialize one of the
// request structs. Here, we're make a requests to retrieve the 6th page of
// featured demon levels of any demon difficulty
let request = LevelsRequest::default()
    .request_type(LevelRequestType::Featured)
    .with_rating(LevelRating::Demon(DemonRating::Hard))
    .page(5);

// To actually issue the request, we call the appropriate method on our Gdcf instance.
// The type parameters on these methods determine how much associated information
// should be retrieved for the request result. Here we're telling GDCF to also
// get us information about the requested levels' custom songs and creators
// instead of just their IDs. "paginate_levels" give us a stream over all pages
// of results from our request instead of only the page we requested.
let stream = gdcf.paginate_levels::<NewgroundsSong, Creator>(request);

// Since we have a stream, we can use all our favorite Stream methods from the
// futures crate. Here we limit the stream to 50 pages of levels a print
// out each level's name, creator, song and song artist.
let future = stream
    .take(50)
    .for_each(|levels| {
        for level in levels {
            match level.custom_song {
                Some(newgrounds_song) =>
                    println!(
                        "Retrieved demon level {} by {} using custom song {} by {}",
                        level.name, level.creator.name, newgrounds_song.name,
                        newgrounds_song.artist
                    ),
                None =>
                    println!(
                        "Retrieved demon level {} by {} using main song {} by {}",
                        level.name, level.creator.name, level.main_song.unwrap().name,
                        level.main_song.unwrap().artist
                    )
            }
        }

        Ok(())
    })
    .map_err(|error| eprintln!("Something went wrong! {:?}", error));

tokio::run(future);
```
