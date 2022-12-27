
number = 10000
open("../benchmark-2022/h-%s.ftd" % number, "w").write("\n\n".join(["-- "
                                                                  "ftd.text: hello "
                                                       "world " + str(i) for
                                                       i in range(number)]))
