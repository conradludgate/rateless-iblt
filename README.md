# rateless-iblt

Rateless invertible bloom lookup table. This is a bloom filter like construction that also exposes lookup support.
It requires no design parameters to tune the correct size, as it naturally resizes until there's enough data to
extract all entries from the table. The intended usecase is set reconcilliation in distributed networks.
This problem involves determining the difference of two sets on different nodes. Rather than sending the entire set,
you can compress it into the bloom lookup table. Later, the receiver can "subtract" their nodes from the lookup table,
which will reveal the differences.

Based on https://arxiv.org/pdf/2402.02668
> Practical Rateless Set Reconciliation
