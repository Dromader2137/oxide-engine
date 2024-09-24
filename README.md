# Oxide engine
This project is a game engine focused on providing easy workflow for creating galaxy scale environments.
## Features
### Two part coordinates
All positions are strored as a vector i64 and a vector f64. The first one counts the chunks (1e12 meters), and the second one specifies the in-chunk position. 
This approach allows the user not to worry about precision issues, which is important if your project can't use a floating origin system for some reason.
### Async mesh loading
When uploading new mesh data to the GPU the engine won't hang untill it's finshed. Insted it'll use the old data untill the new one finishes transferring.
## Contributing
For now, I'am not planning to accept any contributions, although that may change in the future.
