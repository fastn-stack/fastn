
open("../t/html/new-test.ftd", "w").write("\n\n".join(["-- ftd.text: hello "
                                                       "world " + str(i) for
                                                       i in range(1000)]))

f = open("../t/html/new-test.html", "a")
