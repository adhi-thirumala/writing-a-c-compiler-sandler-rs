# switch
## keywords
- switch
- case
- default

## semantic quirks
collect cases

## updates to existing flow
break can break out of both
continue can only be in loop


## execution
effectively an if else
evaluate controlling expression
have a list of case labels
case is a set of conditional jumps
so u take all case values - generate an equal - save that in a variable - and then do jump if not 0
break braks out of inner
and then keep going


statement
| switch (exp condition, statement body)
| case (exp, statement body):
| default (statement body)


<condition>
v = result of condition
for case in cases
equal v and case expr = destination
jump if not zero to case label

if default
jump default

emit body
break label

