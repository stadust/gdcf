# Geometry Dash Caching Framework

GDCF is a combined API wrapper and cache for Geometry Dash. It was written with the idea to provide fast and reliable access to the resources provided by the Geometry Dash servers, falling back to a local cache for often accessed data or if the Geometry Dash servers fail to respond. It allows cached data to  be accessed immediately while triggering cache refreshes asynchronously in the background, making it perferct for usage in environment where response time should be minimized (e.g. pointercrate).

It also attempts to provide a san*er* way of dealing with the Geometry Dash API in general, by being able to automatically glue together multiple requests to provide more complete objects (e.g. a request to `downloadGJLevel` normally only provides the level data and the level's metadata, which includes it's custom song's and creator's *ID*, but nothing more. GDCF can combine this request with a `getGJLevels` automatically to retrieve the custom song data and minimal creator data, or with a `getGJUserInfo` to get the creator's whole profile).

Although written with the mostly insane design decisions made by robtop for the boomlings API in mind, is it possible write APi clients for GDCF interfacing with arbitrary servers, as long as they uphold some invariants about which endpoints provides which information. 

## Disclaimer:
This whole thing is still in very early stages of development. The only documented parts are the structs modelling Geometry Dash objects, and the past few weeks I've rewritten the core gdcf crate at least 3 times. I've only uploaded it here because it was simpler than creating more private git repos on pointercrate itself and because some people I'm working with for an update to pointercrate need it as a reference. If you still want to use this, here's a [discord server](https://discord.gg/sQewUEB)
