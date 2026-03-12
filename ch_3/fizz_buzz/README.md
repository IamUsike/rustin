Challenge 6: FizzBuzz with Twist [Easy]

Classic FizzBuzz 1–100, but also handle multiples of 7 as "Bazz", and for numbers that are multiples of both 3 and 7, print "FizzBazz". Use a match statement (not if-else).

Hints: match on (n % 3, n % 5, n % 7) • use tuples in match arms

| Condition          | Output       |
| ------------------ | ------------ |
| divisible by 3     | Fizz         |
| divisible by 5     | Buzz         |
| divisible by 7     | Bazz         |
| divisible by 3 & 5 | FizzBuzz     |
| divisible by 3 & 7 | FizzBazz     |
| divisible by 5 & 7 | BuzzBazz     |
| divisible by 3,5,7 | FizzBuzzBazz |
| otherwise          | number       |
