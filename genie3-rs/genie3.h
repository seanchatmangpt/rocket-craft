/**
 * @file genie3.h
 * @brief C/C++ Binding declarations for the genie3-rs engine.
 */

#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @struct Vector3
 * @brief A 3D coordinate, vector, or displacement.
 */
typedef struct Vector3 {
    /** The X coordinate or component. */
    float x;
    /** The Y coordinate or component. */
    float y;
    /** The Z coordinate or component. */
    float z;
} Vector3;

/**
 * @struct Rotation3D
 * @brief 3D Euler angles representation for rotations (in degrees).
 */
typedef struct Rotation3D {
    /** Rotation around the Y-axis (in degrees). */
    float pitch;
    /** Rotation around the Z-axis (in degrees). */
    float yaw;
    /** Rotation around the X-axis (in degrees). */
    float roll;
} Rotation3D;

/**
 * @struct Bounds3D
 * @brief Axis-aligned bounding box (AABB) in 3D space.
 */
typedef struct Bounds3D {
    /** The center coordinates of the bounding box. */
    Vector3 center;
    /** The half-extents (half-sizes along each axis) of the bounding box. */
    Vector3 half_extents;
} Bounds3D;

/**
 * @struct Transform
 * @brief Complete placement definition in the 3D world (position, rotation, scale).
 */
typedef struct Transform {
    /** The 3D position vector. */
    Vector3 position;
    /** The 3D Euler rotation angles. */
    Rotation3D rotation;
    /** The 3D scale factors. */
    Vector3 scale;
} Transform;

/**
 * @brief Create a new Vector3.
 * @param x The X coordinate component.
 * @param y The Y coordinate component.
 * @param z The Z coordinate component.
 * @return A new Vector3 structure initialized with the given coordinates.
 */
Vector3 genie3_vector_new(float x, float y, float z);

/**
 * @brief Add two Vector3 vectors.
 * @param a The first vector operand.
 * @param b The second vector operand.
 * @return A new Vector3 containing the element-wise sum of the two vectors.
 */
Vector3 genie3_vector_add(Vector3 a, Vector3 b);

/**
 * @brief Calculate the Euclidean distance between two Vector3 vectors.
 * @param a The first vector.
 * @param b The second vector.
 * @return The straight-line Euclidean distance between the two vectors.
 */
float genie3_vector_distance(Vector3 a, Vector3 b);

#ifdef __cplusplus
}
#endif
