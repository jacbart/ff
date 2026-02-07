## [1.0.10] - 2026-02-07

### 🐛 Bug Fixes

- Esc key with text in input does not cloes immeditaly, added a release script to use git-cliff, added git cliff to nix shell
## [1.0.9] - 2026-02-07

### 🐛 Bug Fixes

- Tui flickering, added in a double buffer

### ⚡ Performance

- More accurate and visually easier to see the matches, worst performance though

### ⚙️ Miscellaneous Tasks

- *(1.0.9)* Updating version in cargo
## [1.0.7] - 2026-02-01

### 🚀 Features

- *(indicator)* Also fix for multiselect auto scrolling, indicator while there is input
## [1.0.6] - 2026-01-31

### 🐛 Bug Fixes

- *(multi-select)* Now can multi-select while filtering
- *(tui)* Sped up file input and selector is followed, removed unused deps and code
## [1.0.5] - 2025-10-25

### 🚀 Features

- *(1.0.5)* Move where tui prints out items
## [1.0.4] - 2025-10-25

### 🚀 Features

- *(1.0.4)* Streaming items

### 🐛 Bug Fixes

- Trimming and checking for empty lines before displaying in tui
## [1.0.3] - 2025-07-19

### 🚀 Features

- *(async)* Updated the search to us quick sort and binary trees plus lsh to group similar items

### 📚 Documentation

- Removed stdin option from readme
## [1.0.2] - 2025-07-12

### 🐛 Bug Fixes

- Selection highlight, removed docs, lock update and version bump
## [1.0.0] - 2025-07-12

### 🚀 Features

- *(v1)* Updates to tui to handle end of terminal and dir as an arg

### 🐛 Bug Fixes

- Fuzzy highlight case insensitive

### 📚 Documentation

- Reorg on the docs again and moved tests out of lib
