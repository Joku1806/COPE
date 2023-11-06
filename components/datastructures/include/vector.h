#pragma once

#include <cassert>
#include <cstring>
#include <utility>

template<typename Vector>
class VectorIterator {
public:
    using ValueType = Vector::ValueType;

    VectorIterator(ValueType* ptr)
        : m_ptr(ptr)
    {
    }

    VectorIterator& operator++()
    {
        m_ptr++;
        return *this;
    }

    VectorIterator operator++(int)
    {
        VectorIterator it = *this;
        ++(*this);
        return it;
    }

    VectorIterator& operator--()
    {
        m_ptr--;
        return *this;
    }

    VectorIterator operator--(int)
    {
        VectorIterator it = *this;
        --(*this);
        return it;
    }

    ValueType& operator[](unsigned index)
    {
        return *(m_ptr + index);
    }

    ValueType* operator->()
    {
        return m_ptr;
    }

    ValueType& operator*()
    {
        return *m_ptr;
    }

    bool operator==(VectorIterator const& other)
    {
        return m_ptr == other.m_ptr;
    }

    bool operator!=(VectorIterator const& other)
    {
        return !(*this == other);
    }

private:
    ValueType* m_ptr;
};

template<typename T>
class Vector {
public:
    using ValueType = T;
    using Iterator = VectorIterator<Vector<T>>

    Vector()
    {
        m_capacity = 0;
        m_size = 0;
        m_data = malloc(0);
    }

    Vector(unsigned capacity)
    {
        capacity = capacity;
        m_size = 0;
        m_data = malloc(capacity * sizeof(T));
    }

    Vector(Vector<T> const& other)
    {
        set_capacity(other.m_capacity);
        m_size = other.m_size;
        memcpy(m_data, other.m_data, m_size * sizeof(T));
    }

    ~Vector()
    {
        clear();
        ::operator delete(m_data, m_capacity * sizeof(T));
    }

    unsigned size() const
    {
        return m_size;
    }

    unsigned capacity()
    {
        return m_size;
    }

    bool is_empty()
    {
        return m_size == 0;
    }

    bool is_full()
    {
        return m_size == m_capacity;
    }

    // TODO: Write an iterator as well!
    T& operator[](unsigned index)
    {
        assert(index < m_size);
        return m_data[index];
    }

    T const& operator[](unsigned index) const
    {
        assert(index < m_size);
        return m_data[index];
    }

    void insert(T const& item, unsigned index)
    {
        assert(index < m_size);
        m_data[index] = item;
    }

    void insert(T&& item, unsigned index)
    {
        assert(index < m_size);
        ::operator new(m_data[index]) T(std::move(item));
    }

    // NOTE: also implement something like EmplaceBack in std::vector?
    void append(T const& item)
    {
        if (needs_to_grow()) {
            set_capacity(next_capacity());
        }

        m_data[m_size++] = item;
    }

    void append(T&& item)
    {
        if (needs_to_grow()) {
            set_capacity(next_capacity());
        }

        ::operator new(m_data[m_size++]) T(std::move(item));
    }

    void swap(unsigned i1, unsigned i2)
    {
        assert(i1 < size);
        assert(i2 < size);

        T tmp = data[i1];
        data[i1] = data[i2];
        data[i2] = tmp;
    }

    T remove(unsigned index)
    {
        assert(index < size);

        T removed = data[index];
        data[index] = data[--v->size];

        if (needs_to_shrink()) {
            set_capacity(m_size);
        }

        return removed;
    }

    void clear()
    {
        for (unsigned i = 0; i < m_size; i++) {
            m_data[i].~T();
        }

        m_size = 0;
    }

    Iterator begin()
    {
        return Iterator(m_data);
    }

    Iterator end()
    {
        return Iterator(m_data + m_size);
    }

private:
    constexpr unsigned resize_factor = 1.25;

    unsigned m_capacity = 0;
    unsigned m_size = 0;
    T* m_data = nullptr;

    bool needs_to_shrink()
    {
        // NOTE: I don't know if this is the right approach.
        // We want to aggresively shrink large allocations,
        // but this would disproportionally affect small vectors,
        // because less removes are needed to fall below the resize factor.
        // Maybe we should add a lower bound where it doesn't matter,
        // if we shrink.
        return m_size > 0 && m_capacity / m_size > resize_factor;
    }

    bool needs_to_grow()
    {
        return m_size == m_capacity;
    }

    unsigned next_capacity()
    {
        return m_capacity < 2 ? 2 : m_capacity * resize_factor;
    }

    void set_capacity(unsigned capacity)
    {
        assert(capacity >= size);

        T* data = (T*)::operator new(capacity * sizeof(T));
        for (unsigned i = 0; i < m_size; i++) {
            ::operator new(&data[i]) T(std::move(m_data[i]));
            m_data[i].~T();
        }
        ::operator delete(m_data, m_capacity * sizeof(T));

        m_data = data;
        m_capacity = capacity;
    }
};
