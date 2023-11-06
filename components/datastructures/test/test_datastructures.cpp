#include "unity.h"
#include "vector.h"

TEST_CASE("Creating an empty vector", "[vector]")
{
    Vector<int> v;
    TEST_ASSERT_EQUAL(v.size(), 0)
}

TEST_CASE("Creating a vector with a specific capacity", "[vector]")
{
    Vector<int> v = Vector(10);

    TEST_ASSERT_EQUAL(v.size(), 0);
    TEST_ASSERT_EQUAL(v.capacity(), 10);
}

TEST_CASE("Get an item from a vector", "[vector]")
{
    Vector<int> v;
    v.append(42);

    TEST_ASSERT_EQUAL(v[0] == 42);
}

TEST_CASE("Get an item from a vector out of bounds", "[vector][fails]")
{
    Vector<int> v;
    unsigned invalid = v[3];
}

TEST_CASE("Insert an item at a specific index", "[vector]")
{
    Vector<int> v = Vector(5);

    int items[5] = { 3, 87, 2, 25, 48 };
    for (unsigned i = 0; i < 5; i++) {
        v.append(items[i]);
    }

    v.insert(60, 3);
    v.insert(10, 1);

    int items_after[5] = { 3, 10, 2, 60, 48 };
    for (unsigned i = 0; i < 5; i++) {
        TEST_ASSERT_EQUAL(v[i], items_after[i]);
    }
}

TEST_CASE("Append multiple items", "[vector]")
{
    int items[10] = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 };
    Vector<int> v;

    for (unsigned i = 0; i < 10; i++) {
        v.append(items[i]);
        TEST_ASSERT_EQUAL(v[i], items[i]);
    }
}

TEST_CASE("Swap two items", "[vector]")
{
    Vector<int> v = Vector(5);

    int items[5] = { 3, 87, 2, 25, 48 };
    for (unsigned i = 0; i < 5; i++) {
        v.append(items[i]);
    }

    v.swap(v, 1, 4);
    v.swap(v, 2, 3);

    int items_after[5] = { 3, 48, 25, 2, 87 };
    for (unsigned i = 0; i < 5; i++) {
        TEST_ASSERT_EQUAL(v[i], items_after[i]);
    }
}

Test(vector, empty)
{
    int items[10] = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 };
    int_vector_t* v = int_vector_create();
    cr_assert(int_vector_empty(v));

    for (unsigned i = 0; i < 10; i++) {
        int_vector_append(v, items[i]);
        cr_assert(!int_vector_empty(v));
    }

    int_vector_clear(v);
    cr_assert(int_vector_empty(v));

    int_vector_destroy(v);
}

Test(vector, full)
{
    int items[10] = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 };
    int_vector_t* v = int_vector_create_with_capacity(10);

    for (unsigned i = 0; i < 10; i++) {
        int_vector_append(v, items[i]);
    }

    cr_assert(int_vector_full(v));
    int_vector_destroy(v);
}

Test(vector, remove)
{
    int items[10] = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 };
    int_vector_t* v = int_vector_create();

    for (unsigned i = 0; i < 10; i++) {
        int_vector_append(v, items[i]);
    }

    cr_assert(int_vector_remove(v, 7) == items[7]);
    cr_assert(int_vector_remove(v, 3) == items[3]);
    cr_assert(int_vector_remove(v, 0) == items[0]);
    cr_assert(int_vector_size(v) == 7);

    int items_after[7] = { 9, 1, 2, 8, 4, 5, 6 };

    for (unsigned i = 0; i < 7; i++) {
        cr_assert(int_vector_at(v, i) == items_after[i]);
    }

    int_vector_destroy(v);
}

Test(vector, remove_invalid, .signal = SIGABRT)
{
    int_vector_t* v = int_vector_create();
    int_vector_remove(v, 13);
    int_vector_destroy(v);
}

Test(vector, clear)
{
    int_vector_t* v = int_vector_create();
    int_vector_append(v, 33);
    int_vector_clear(v);
    cr_assert(int_vector_size(v) == 0);
}

Test(vector, clone)
{
    int items[10] = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9 };
    int_vector_t* v = int_vector_create();

    for (unsigned i = 0; i < 10; i++) {
        int_vector_append(v, items[i]);
    }

    int_vector_t* cloned = int_vector_clone(v);

    cr_assert(v->capacity == cloned->capacity);
    cr_assert(v->size == cloned->size);

    for (unsigned i = 0; i < int_vector_size(v); i++) {
        cr_assert(int_vector_at(v, i) == int_vector_at(cloned, i));
    }
}

Test(vector, destroy)
{
    int_vector_t* v = int_vector_create();
    int_vector_destroy(v);
}

Test(vector, destroy_invalid, .signal = SIGABRT)
{
    int_vector_t* v = int_vector_create();
    v->size = 9001;
    int_vector_destroy(v);
}

typedef struct test_struct {
    int a;
    void* b;
    char d;
} test_struct_vector;

MAKE_SPECIFIC_VECTOR_HEADER(test_struct_vector, test_struct_vector)
MAKE_SPECIFIC_VECTOR_SOURCE(test_struct_vector, test_struct_vector)

Test(vector, generic_create)
{
    test_struct_vector_vector_t* v = test_struct_vector_vector_create();
    cr_assert(test_struct_vector_vector_size(v) == 0);
    test_struct_vector_vector_destroy(v);
}
