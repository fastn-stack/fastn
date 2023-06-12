# Server Side Rendering

We have to send fully rendered HTML to crawlers. Since in `fastn build` we can not serve different content based on
user agent, the HTML we send must include server rendered HTML.

Once browser loads server rendered content, we can either nuke it and recreate the entire DOM, but it may cause flicker
etc, so ideally we should re-use the DOM and "hydrate it" by attaching event handlers.

To do server side rendering we can continue our JS approach, and use or create a `jsdom` like library, which keeps 
track of event handlers to each DOM node. Dom nodes will be identified by unique IDs (we can keep a global integer as
dom ID and keep incrementing it whenever a new dom node is constructed).

When an event handler is attached the server side jsdom will store it in a map of id to event spec, and it will be 
handed over to the page.
