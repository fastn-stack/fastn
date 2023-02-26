import random
import string
import json

loop_body_template = """

-- import: fastn/processors


-- component check-loop:
string list cities:

-- ftd.column:

-- ftd.text: $obj
$loop$: $check-loop.cities as $obj


-- end: ftd.column

-- end: check-loop

"""

list_loop_calling = """

-- string list cities:
$processor$: processors.get-data

{0}


-- check-loop:
cities: $cities

"""


def str_list_data(str_length, list_length):
    arr = []
    for idx in range(0, list_length):
        arr.append(''.join(
            random.choices(string.ascii_lowercase + string.digits,
                           k=str_length)))
    return arr


def create_list_str_loop_document(str_length, list_length):
    data = str_list_data(str_length, list_length)
    json_data = json.dumps(data)
    print(len(json_data))
    template_file = loop_body_template + list_loop_calling.format(json.dumps(data))
    with open("../loop.ftd", 'w') as f:
        f.write(template_file)


def int_list_data(list_length):
    rand_list = []
    for i in range(list_length):
        rand_list.append(random.randint(1, 99999999999))
    return rand_list


def create_list_int_loop_document(str_length, list_length):
    data = str_list_data(str_length, list_length)
    json_data = json.dumps(data)
    print(len(json_data))
    template_file = loop_body_template + list_loop_calling.format(json.dumps(data))
    with open("../loop.ftd", 'w') as f:
        f.write(template_file)


if __name__ == "__main__":
    import sys

    list_len = 1000
    str_len = 10

    if len(sys.argv) > 2:
        list_len = int(sys.argv[1])
        str_len = int(sys.argv[2])

    if len(sys.argv) > 1:
        list_len = int(sys.argv[1])

    create_list_str_loop_document(str_len, list_len)
