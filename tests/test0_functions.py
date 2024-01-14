
def test():
    return True

def multiply(n, factor):
    print n
    print factor
    result = n * factor
    return result

if test() == True:
    print "test(): True"
else:
    print "test(): False"

result = multiply(10, 2)
print result
result
