# cycler
A simultainious write/read data structure.

This is a data synchronization system with 1 writer and N readers. All writers and readers can operate on the data at the same time and will access the most up to date version when they switch.
This is accomplished by storing N + 2 copies of the data with the writer publishing new versions of the data.
The purpose of this is to allow read threads to have an unchanging up to date copy of the main data while the writer can edit it at the same time.
## Requirements
The data type must also implement both `ReadAccess` and `WriteAccess` otherwise the access functions won't be available.
These types allow the reader to see a subset of the data, intended for hiding data only the writer needs like a change log for optimization of the copy.
If you don't care about access restrictions you can set `ReadAccess::Read` and `WriteAccess::Write` to Self and there will be zero runtime cost as the compiler should optimize that out.
## Optimization
The most optimal thing to do with regard to memory usage is to have a single reader.
N + 2 copies of the data must exist because in the worst case scenario all N readers are reading separate copies and the writer is finalizing a write.
In this case the N copies of the data that are being read are inaccessible to writes and the last updated index cannot be written to so there must be an additional copy.
What this means is that if you only utilize a single writer and distribute that the minimum amount of memory will be used.
The trade off is that if you have multiple reading loops that operate at differing rates all the loops will operate at the slowest speed.
Adding readers does not increase the amount of data to copy and may only slightly increase the time between copy switches.
This also is based on the `clone_from` idea to clone values which is not implemented by derive normally (Derivative can auto derive for you).
This is a major optimization chance in this case and you can test/track the changes to reduce copy time.
