## [0.3.1] - 2018-08-14

### Removed
- Removed optional `clippy` dependency

## [0.3.0] - 2017-04-14

### Removed
- Removed `VecMemory` struct

### Added
- Added `AlignVec` struct
- Added `BoxMemory` struct
- Added `Pointers::mark_ptr` method

### Changed
- Removed `unsafe` requirement from `Memory::allocate` method
- Renamed `Hazard` struct to `Pointers`

### Fixed
- Fixed correctness of `Pointers::mark` method

## [0.2.0] - 2017-03-24

### Changed
- Added `threshold` parameter to `Hazard` constructor
- Relaxed atomic orderings

## [0.1.0] - 2017-03-01
- Initial release
