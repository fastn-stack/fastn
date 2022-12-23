# FPM Design

Download art from https://github.com/vitiral/artifact/releases/tag/1.0.1

Unzip the `art` binary and move to ~/bin/. Ensure `.zshrc` says 
`export PATH=$PATH:$HOME/bin`.

On Mac if you get error from OS, locate the file in "Finder", and then right click
and select "Open", it will show a warning and "Open" button, click Open and close the
newly launched terminal. Go back to shell and run `art` and it will work now.

Run `art serve`.

If you add a new file, you will have to restart the `art serve` server and reload
the page in browser.