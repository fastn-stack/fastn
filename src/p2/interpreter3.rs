/*

Import Structure
----------------
- main.ftd

- main.ftd
    - a1.ftd
        - a2.ftd

- main.ftd
    - a1.ftd
        - a2.ftd
    - a3.ftd
        - a2.ftd
        - a4.ftd
            - a5.ftd


Points to consider
 0. After seeing `import` section interpreter will return with state StuckOnImport(file)
 1. There will a global stack which will contain the all ancestor files, which are not fully
 interpreted yet
 2. With global stack we need to store the section number, till where it has been interpreted
 3. We will push items into global stack after when we will start interpreting it
 4. Now question is who will pop the items from stack and when do we start interpreting ancestor
 file again.
 5. Before returning at Stuck* interpreter need to store at it's current state and need to make
 sure it will start again from where it stopped.


Method Signature

InterpreterState:-
    Properties:
        stack
        bag for dependencies

    continue_(file):
        continue interpreting
        stuckOnImport
            - save state of file into stack
            - return with current state
        pop element last document from stack(ancestor files) and continue interpreting (No need
        for a loop)
            - recursive call to continue_

    continue_after_import(source of file)
        parse source file
        call continue
*/
