from itertools import combinations
from itertools import product


def factorial_combination(arr):
    result = []
    for i in range(1, len(arr)+1):
        comb = combinations(arr, i)
        for j in list(comb):
            result.append("".join(j))
    return result


def factorial_combination(arr):
    return ["".join(i) for i in product(arr, repeat=len(arr))]


# TODO: Let's do it later, now we will do it manually
def generate_sitemap(len):
    arr = [chr(97+x) for x in range(len)]
    print(arr)
    fact = factorial_combination(['a', 'b', 'c'])
    print(fact)

generate_sitemap(5)