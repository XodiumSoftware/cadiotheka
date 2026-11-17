# docs/

Landing page for the published rustdoc site
(<https://xodiumsoftware.github.io/cadiotheka/>), plus the page stylesheet.

`index.html` is the only hand-written file here: `--enable-index-page` is a
nightly-only rustdoc flag, so on stable the workflow copies this page to the
root of the assembled site instead. It links to each crate's generated docs.

Edit `index.html` when a crate is added to or removed from the workspace.
