# Geometry Dash Caching Framework

GDCF is a combined API wrapper and cache for Geometry Dash. It was written with the idea to provide fast and reliable access to the resources provided by the Geometry Dash servers, falling back to a local cache for often accessed data or if the Geometry Dash servers fail to respond. It allows cached data to  be accessed immediately while triggering cache refreshes asynchronously in the background, making it perferct for usage in environment where response time should be minimized (e.g. pointercrate).

It also attempts to provide a san*er* way of dealing with the Geometry Dash API in general, by being able to automatically glue together multiple requests to provide more complete objects (e.g. a request to `downloadGJLevel` normally only provides the level data and the level's metadata, which includes it's custom song's and creator's *ID*, but nothing more. GDCF can combine this request with a `getGJLevels` automatically to retrieve the custom song data and minimal creator data, or with a `getGJUserInfo` to get the creator's whole profile).

Although written with the mostly insane design decisions made by robtop for the boomlings API in mind, is it possible write APi clients for GDCF interfacing with arbitrary servers, as long as they uphold some invariants about which endpoints provides which information. 

## Disclaimer:
This whole thing is still in very early stages of development. The only documented parts are the structs modelling Geometry Dash objects, and the past few weeks I've rewritten the core gdcf crate at least 3 times. I've only uploaded it here because it was simpler than creating more private git repos on pointercrate itself and because some people I'm working with for an update to pointercrate need it as a reference. If you still want to use this, here's a [discord server](https://discord.gg/sQewUEB)


## Example:
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
