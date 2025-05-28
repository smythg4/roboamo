# RoboAMO
## Thoughts for future:
- Consider non-exclusive quals
- Improve command line interface (custom file selection, custom output options)
- Consider stand alone GUI
- Consider web interface (who hosts it, how is IO handled)
- Do ASM expiration dates matter? If so, just ignore them or add them to a Qual struct?
- Add scoring for individual teams (eg Day Check is highest pri, Det 1 next, ...)

## Known Bugs
- look at logic for updating database in the event that file was deleted
- update parsing strategy for ASM names (right now depends on '  ' before rate). Can use regex for better matches.
- need to do a better job cleaning strings for random characters ( , : etc)



## 🤝 Contributing

### Clone the repo

```bash
git clone https://github.com/smythg4/roboamo@latest
cd roboamo
```