<div align="center">
<h1> Pages </h1>
<p>

**Auto-deploy static websites from git repositories**

</p>


</div>

## Why?

SSHing into a server and performing a `git pull` is boring. I couldn't
find any free(as in freedom) software for automating static website
deploys like GitHub Pages or Netlify.

This is very minimal, all it does is a `git fetch $branch` but it works
for me :)

## Usage

1. All configuration is done through
   [./config/default.toml](./config/default.toml)(can be moved to
   `/etc/static-pages/config.toml`). Too add a website,
   make a similar entry:

    ```toml
    pages = [
    	{ branch = "gh-pages", repo = "https://github.com/realaravinth/realaravinth/", path ="/var/www/pages/realaravinth", secret = "mytopsecretsuperlongpassword123" },
    ]
    ```

2. If `pages` is deployed at `pages.example.com` and you wish to deploy
   changes from `gh-pages` branch, you can do so with the following
   command:
    ```bash
     curl -v --location --request POST 'https://pages.example.com/api/v1/update' \
    --header 'Content-Type: application/json' \
    --data-raw "{
    	\"secret\": \"$token\",
    	\"branch\": \"gh-pages\"
    }"
    ```
