open("../t/html/new-test.ftd", "w").write("\n\n".join(["-- ftd.text: hello world " + str(i) for i in range(45)]))

f = open("../t/html/new-test.html", "a")
