last_function = """
-- component check-performance{0}:

-- ftd.text: Hello World {0}

-- end: check-performance{0}

"""

body_template = """

-- component check-performance{0}:

-- ftd.column:

-- ftd.text: Hello World {0}

-- check-performance{1}:

-- end: ftd.column

-- end: check-performance{0}

"""

function_calling = """

-- check-performance1:

"""


def generate_component_calling(number):
    generated_ftd_template = last_function.format(number)

    while number > 1:
        number = number - 1
        generated_ftd_template += body_template.format(number, number + 1)

    generated_ftd_template += function_calling
    return generated_ftd_template


def create_component_calling_document(no_fun_call):
    template_file = generate_component_calling(no_fun_call)
    with open("../component-calling.ftd", 'w') as f:
        f.write(template_file)


if __name__ == "__main__":
    import sys
    number = 1000
    if len(sys.argv) > 1:
        number = int(sys.argv[1])
    create_component_calling_document(number)
