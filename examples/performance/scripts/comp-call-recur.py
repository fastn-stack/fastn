template = """

-- component check-performance:
integer break:

-- ftd.column:

-- ftd.text: Hello World

-- ftd.integer: $check-performance.break

-- check-performance:
break: $decrement(a=$check-performance.break)
if: {check-performance.break != 0}

-- end: ftd.column

-- end: check-performance


-- integer decrement(a):
integer a:

a - 1

"""

function_call = """

-- check-performance:
break: {0}

"""


def create_component_rec_call_document(no_fun_call):
    template_file = template + function_call.format(no_fun_call)
    with open("../comp-rec-call.ftd", 'w') as f:
        f.write(template_file)


if __name__ == "__main__":
    import sys

    no_fun_call = 1000
    if len(sys.argv) > 1:
        no_fun_call = int(sys.argv[1])
    create_component_rec_call_document(no_fun_call)
