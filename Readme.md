# `pst`: show running processes in context

`pst` alone will print the list of running processes in a tree structure.  Pass
`-d` for more detail or `-l` for less (which makes the COMMAND column longer).

You can also pass a query string.  Queries match literally against the entire
command (this usually includes the process' arguments) or its PID.  The entire
tree is printed, but matching lines are highlighted.

# History

    ps
    # Oh, right.
    ps -e
    ps -eh
    # No, that's not it.
    man ps
    ps -eHf
    ps -eHf | pyth
    # Oops.
    ps -eHf | grep pyth

`pst` started out life as an alias for something like `ps -eHf`, though I think
I had some more options in there.  And then it grew.
