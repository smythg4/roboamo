# RoboAMO - Automated Military Personnel Assignment System
A high-performance Rust application for optimizing Navy personnel assignments based on qualifications, duty status, and operational requirements. Processes entire unit manning in under 25ms.
Overview
RoboAMO solves the complex problem of assigning qualified personnel to team positions while considering:

- Qualification requirements and scarcity
- Projected Rotation Dates (PRD)
- Duty status (TAR/SELRES)
- Rate/rank constraints
- Multi-factor optimization scoring

## Features

- Lightning Fast: Core algorithm runs in ~2ms, full database update in ~22ms
- Smart Assignment: Prioritizes based on PRD, qualification scarcity, and duty status
- Excel Integration: Parses Navy personnel data from XLSX files
- Bottleneck Analysis: Identifies qualification shortages and manning gaps
- Visual Output: Color-coded results showing assignment priority
- Database Persistence: SQLite backend for data storage

## Installation
#### Clone the repository
```
git clone https://github.com/yourusername/roboamo.git
cd roboamo
```

### Build the optimized binary
```
cargo build --release
Usage
Basic Usage
bash# Run assignments using existing database
./target/release/roboamo
```

### Update database from Excel files and run assignments
To use default file paths.
```
./target/release/roboamo -u
```

To use custom filepaths, check:
```
./targer/release/roboamo --help
```

### Data Files Required
Place these files in the data/ directory:

- PeopleMaster.xlsx - Personnel data export with two worksheets:

- FLTPMS - Contains names and PRD dates
- ASM Report - Contains personnel qualifications


- qualtable.csv - Maps ASM qualification names to friendly names
```
ASM Name,Friendly Name
ADMIN AND LOGISTICS MANAGEMENT,TAO
MISSION COMMANDER,MC
```

- teams.csv - Team position requirements
```
Team Name,Qualification,Count
Operations,TAO,2
Operations,MC,1
Maintenance,CDI,3
```

## Output
The system provides:
### Supply/Demand Analysis
```
=== Supply/Demand Analysis ===
CDI             15 / 12 = 1.25  ⚠️  TIGHT
QAR              8 / 10 = 0.80  ❌ SHORTAGE
TAO             25 / 15 = 1.67  ✅ OK
```
### Team Assignments
```
Operations:
  🟢 Smith, John (150) as TAO
  🟡 Jones, Mary (2500) as MC
  ❌ UNFILLED: 1 QAR position(s)
```
### Results File
Saves detailed assignments to data/assignments.csv
### Performance
- Benchmarked on ~130 personnel across 10 teams:
- Core Algorithm 2.1ms 
- With DB Update 21.4ms

### Optimizations include:
- Lazy static regex compilation
- Zero-copy string handling with ```Cow<str>```
- Batched database transactions
- Efficient data structures

## Architecture
```
roboamo/
├── src/
│   ├── main.rs           # CLI and assignment algorithm
│   ├── lib.rs            # Core data structures
│   ├── asm_parser.rs     # Excel parsing logic
│   ├── csv_funcs.rs      # CSV I/O operations
│   └── database.rs       # SQLite operations
├── data/ (not in repo)
│   ├── PeopleMaster.xlsx # Personnel data
│   ├── qualtable.csv     # Qualification mappings
│   └── teams.csv         # Team requirements
└── Cargo.toml
```

## Algorithm
The assignment system uses a greedy algorithm with multi-factor scoring:

* Qualification Criticality: Assigns scarce qualifications first
* Personnel Scoring: Considers:

- Duty status (SELRES personnel scored higher for availability)
- Time until PRD (personnel leaving soon scored higher)
- Qualification diversity (avoid using multi-qualified personnel for common positions)
- High-demand qualification preservation

## Building from Source
Requires Rust 1.70+ and the following dependencies:

calamine - Excel file parsing
chrono - Date handling
clap - CLI argument parsing
csv - CSV file operations
regex - Pattern matching
once_cell - Lazy static initialization
rusqlite - SQLite database

Author
Nerd

## Thoughts for future:
- Consider non-exclusive quals
- Consider stand alone GUI
- Consider web interface (who hosts it, how is IO handled)
- Do ASM expiration dates matter? If so, just ignore them or add them to a Qual struct?
- Add scoring for individual teams (eg Day Check is highest pri, Det 1 next, ...)

## Known Bugs
- update parsing strategy for ASM names (right now depends on '  ' before rate). Can use regex for better matches.
- need to do a better job cleaning strings for random characters ( , : etc)