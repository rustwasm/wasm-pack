# Release Checklist

This is a list of the things that need to happen during a release.

1. Open the associated milestone. All issues and PRs should be closed. If
  they are not you should reassign all open issues and PRs to future
  milestones.
1. Go through the commit history since last release. Ensure that all PRs
  that have landed are marked with the milestone. You can use this to 
  show all the PRs that are merged on or after YYY-MM-DD: 
  `https://github.com/issues?utf8=%E2%9C%93&q=repo%3Arustwasm%2Fwasm-pack+merged%3A%3E%3DYYYY-MM-DD`
1. Go through the closed PRs in the milestone. Each should have a changelog
  label indicating if the change is docs, fix, feature, or maintenance. If
  there is a missing label, please add one.
1. Choose an emoji for the release. Try to make it some sort of transition
  from the previous releases emoji (point releases can be a little weirder).
1. Create a new branch "#.#.#" where "#.#.#" is the release's version.
1. Add this release to the `CHANGELOG.md`. Use the structure of previous 
  entries.
1. Update `DEFAULT_CHROMEDRIVER_VERSION` in `chromedriver.rs`. 
  Version is the response of `https://chromedriver.storage.googleapis.com/LATEST_RELEASE`.
1. Update `DEFAULT_GECKODRIVER_VERSION` in `geckodriver.rs`.
  Version is the name of the latest tag - `https://github.com/mozilla/geckodriver/releases/latest`.
1. Update the version in `Cargo.toml`.
1. Update the version number and date in `docs/index.html`.
1. Run `cargo update`.
1. Run `cargo test`.
1. Run `cargo build`.
1. Copy `README.md` to `npm/README.md`
1. Bump the version number in `npm/package.json`
1. `cd npm && npm install`
1. Push up a commit with the `Cargo.toml`, `Cargo.lock`, `docs/index.html`,
  and `CHANGELOG.md` changes. The commit message can just be "#.#.#".
1. Request review from `@ashleygwilliams` and `@drager`.
1. `git commit --amend` all changes into the single commit.
1. Once ready to merge, tag the commit with the tag `v#.#.#`.
1. Wait for the CI to go green.
1. The CI will build the release binaries. Take the `CHANGELOG.md` release
  entry and cut and paste it into the release body.
1. Be sure to add any missing link definitions to the release.
1. Hit the big green Merge button.
1. `git checkout master` and `git pull --rebase origin master`
1. Run `cargo test`.
1. `cargo publish`
1. `cd npm && npm publish`
1. Tweet.
