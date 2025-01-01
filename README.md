# Netdog - Easy HTTP Server
- Need a quick and dirty HTTP server? No problem!
- Want to create static web pages? No problem!
- Want to create client side webapps? No problem!

Netdog can do all of that and more!

## Setting up
If you're on windows you can download Netdog from the releases page.

Or (for other platforms), you can build it from source.

After installing, create your config toml file:

```toml
ip = "127.0.0.1"                            # REQUIRED | IP.
port = 5000                                 # OPTIONAL | Port. Defaults to 8080.

[logger]                                    # OPTIONAL | Logger configuration.
print = true                                # OPTIONAL | Whether to print or not. Defaults to true.
log_file = 'logfile.log'                    # OPTIONAL | File to log to. If not specified, netdog will not log to a file.

[routes.main]                               # New Route -> "main" | Name must be unique, but is not important.
methods = ["GET"]                           # OPTIONAL | List of methods (GET, POST).
url = "/"                                   # REQUIRED | Url to access.
path = "mainpage.html"                      # REQUIRED | Path to serve from.

# Make sure that routes with '*' come last
[routes.resources]                          # New Route -> "resources" | Name must be unique, but is not important.
methods = ["GET"]                           # OPTIONAL | List of methods (GET, POST).
url = "/r/*"                                # REQUIRED | Url to access. '*' means anything can come after that.
path = "/resources/*"                       # REQUIRED | Path to serve from. '*' means that the '*' part of the url gets inserted here.

[errors.404]                                # OPTIONAL | Route for Error 404's.
path = "errors/error_404.html"              # REQUIRED | Path to serve from.
```

Then run `netdog my-config.toml` or `netdog` (config file path defaults to *config.toml*)
