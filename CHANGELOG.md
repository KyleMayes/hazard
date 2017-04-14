## [0.3.0] - UNRELEASED

### Removed
- Removed `VecMemory` struct

### Added
- Added `AlignVec` struct
- Added `BoxMemory` struct

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
